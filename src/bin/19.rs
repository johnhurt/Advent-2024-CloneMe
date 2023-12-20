use std::{collections::HashMap, ops::Range};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, anychar, i64},
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, preceded, tuple},
    IResult,
};
use strum_macros::FromRepr;

advent_of_code::solution!(19);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Part {
    x: i64,
    m: i64,
    a: i64,
    s: i64,
}

impl Part {
    // Construct a part from a vec of props in any order (for deserialization)
    fn from_props(props: Vec<(Prop, i64)>) -> Self {
        use Prop as P;
        let mut result = Part::default();

        props.into_iter().for_each(|(p, v)| match p {
            P::X => result.x = v,
            P::M => result.m = v,
            P::A => result.a = v,
            P::S => result.s = v,
        });

        result
    }

    // Get the value for the given xmas prop
    fn get(&self, prop: Prop) -> i64 {
        use Prop as P;

        match prop {
            P::X => self.x,
            P::M => self.m,
            P::A => self.a,
            P::S => self.s,
        }
    }

    // Sum all the properties for this part
    fn sum(self) -> i64 {
        let Part { x, m, a, s } = self;

        x + m + a + s
    }
}

/// For part 2 we deal with ranges of parts in stead of individuals. This
/// represents a set of parts with bands of values for each xmas prop
#[derive(Debug, Clone)]
struct PartRange {
    x: Range<i64>,
    m: Range<i64>,
    a: Range<i64>,
    s: Range<i64>,
}

impl PartRange {
    fn get(&self, prop: Prop) -> &'_ Range<i64> {
        use Prop as P;

        match prop {
            P::X => &self.x,
            P::M => &self.m,
            P::A => &self.a,
            P::S => &self.s,
        }
    }

    fn set(&mut self, prop: Prop, range: Range<i64>) {
        use Prop as P;

        match prop {
            P::X => self.x = range,
            P::M => self.m = range,
            P::A => self.a = range,
            P::S => self.s = range,
        }
    }

    // Split the this range according to the rules for the given operator at the
    // given value. Here are some examples
    // 0..10 gt 6 -> 0..7 & 7..10
    // 0..10 lt 6 -> 0..6 & 6..10
    // 0..10 gt 10 -> 0..11 & 11..
    // 0..10 lt 10 -> 0..10 & 10..
    fn split_at(
        &self,
        prop: Prop,
        op: Op,
        mut val: i64,
    ) -> (Option<Self>, Option<Self>) {
        if op == Op::Gt {
            val += 1;
        }
        let range = self.get(prop).clone();
        if range.contains(&val) {
            let left_range = range.start..val;
            let right_range = val..range.end;
            let mut left = self.clone();
            let mut right = self.clone();
            left.set(prop, left_range);
            right.set(prop, right_range);
            (Some(left), Some(right))
        } else if val == range.end && op == Op::Lt {
            let left_range = range.start..val;
            let right_range = val..(val + 1);
            let mut left = self.clone();
            let mut right = self.clone();
            left.set(prop, left_range);
            right.set(prop, right_range);
            (Some(left), Some(right))
        } else if val < range.start {
            (Some(self.clone()), None)
        } else {
            (None, Some(self.clone()))
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
enum Prop {
    X = b'x',
    M = b'm',
    A = b'a',
    S = b's',
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Workflow<'a> {
    Named(&'a str),
    Accept,
    Reject,
}

impl<'a> Workflow<'a> {
    fn parse(input: &'a str) -> Self {
        match input.as_bytes()[0] {
            b'A' => Workflow::Accept,
            b'R' => Workflow::Reject,
            _ => Workflow::Named(input),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
enum Op {
    Gt = b'>',
    Lt = b'<',
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkflowRule<'a> {
    Conditional {
        lhs: Prop,
        op: Op,
        rhs: i64,
        target: Workflow<'a>,
    },
    Default(Workflow<'a>),
}

impl<'a> WorkflowRule<'a> {
    /// Evaluate this workflow rule and return the target workflow it is sent
    /// to if any
    fn apply(&self, part: Part) -> Option<Workflow<'a>> {
        use WorkflowRule as W;

        match self {
            W::Conditional {
                lhs,
                op,
                rhs,
                target,
            } => match op {
                Op::Gt => (part.get(*lhs) > *rhs).then_some(*target),
                Op::Lt => (part.get(*lhs) < *rhs).then_some(*target),
            },
            W::Default(wf) => Some(*wf),
        }
    }

    /// Apply this rule to a range of workflows. The result is a split of a
    /// range of parts that are sent to the target workflow and a range that
    /// are not
    fn apply_ranged(
        &self,
        parts: &PartRange,
    ) -> (Option<PartRange>, Option<(Workflow<'a>, PartRange)>) {
        use WorkflowRule as W;

        match self {
            W::Conditional {
                lhs,
                op,
                rhs,
                target,
            } => {
                let (left, right) = parts.split_at(*lhs, *op, *rhs);
                match op {
                    Op::Gt => (left, right.map(|range| (*target, range))),
                    Op::Lt => (right, left.map(|range| (*target, range))),
                }
            }
            W::Default(wf) => (None, Some((*wf, parts.clone()))),
        }
    }
}

fn parse_workflow_rule(input: &str) -> IResult<&'_ str, WorkflowRule<'_>> {
    alt((
        map(
            tuple((
                anychar::<&'_ str, _>,
                anychar::<&'_ str, _>,
                i64,
                preceded(tag(":"), alpha1),
            )),
            |(lhs, op, rhs, target)| WorkflowRule::Conditional {
                lhs: Prop::from_repr(lhs as u8).unwrap(),
                op: Op::from_repr(op as u8).unwrap(),
                rhs,
                target: Workflow::parse(target),
            },
        ),
        map(alpha1, |target| {
            WorkflowRule::Default(Workflow::parse(target))
        }),
    ))(input)
}

fn parse_workflow(line: &str) -> (Workflow<'_>, Vec<WorkflowRule>) {
    let result: IResult<_, _> = tuple((
        map(alpha1, Workflow::Named),
        delimited(
            tag("{"),
            separated_list1(tag(","), parse_workflow_rule),
            tag("}"),
        ),
    ))(line);

    result.unwrap().1
}

fn parse_prop(prop: &str) -> IResult<&'_ str, (Prop, i64)> {
    tuple((
        map(anychar, |c| Prop::from_repr(c as u8).unwrap()),
        preceded(tag("="), i64),
    ))(prop)
}

fn parse_parts(line: &str) -> Part {
    let result: IResult<_, _> = map(
        delimited(tag("{"), separated_list1(tag(","), parse_prop), tag("}")),
        Part::from_props,
    )(line);

    result.unwrap().1
}

/// Run the workflow on the given part and return true if it was ultimately
/// accepted
fn run_workflows(
    part: Part,
    workflows: &HashMap<Workflow<'_>, Vec<WorkflowRule<'_>>>,
) -> bool {
    let mut curr = Workflow::Named("in");

    while let Some(workflow) = workflows.get(&curr) {
        for rule in workflow.iter() {
            if let Some(target) = rule.apply(part) {
                curr = target;
                break;
            }
        }
    }

    curr == Workflow::Accept
}

/// Run the workflow on the complete range of workflows and return the
/// total number of possible parts that are accepted
fn run_ranged_workflows(
    range: PartRange,
    workflows: &HashMap<Workflow<'_>, Vec<WorkflowRule<'_>>>,
) -> i64 {
    let mut to_process = vec![(Workflow::Named("in"), range)];
    let mut result = 0;

    while let Some((curr_wf, mut parts)) = to_process.pop() {
        if let Some(workflow) = workflows.get(&curr_wf) {
            for rule in workflow.iter() {
                let (kept_opt, sent_opt) = rule.apply_ranged(&parts);

                // Push the part ranges that are sent to the work queue
                if let Some(sent) = sent_opt {
                    to_process.push(sent);
                }

                // Keep processing any ranges still in the workflow
                if let Some(kept) = kept_opt {
                    parts = kept;
                }
            }
        }

        if curr_wf == Workflow::Accept {
            let PartRange { x, m, a, s } = parts;
            result += (x.end - x.start)
                * (m.end - m.start)
                * (a.end - a.start)
                * (s.end - s.start);
        }
    }

    result
}

pub fn part_one(input: &str) -> Option<i64> {
    let mut regions = input.split("\n\n");
    let workflows = regions
        .next()
        .unwrap()
        .lines()
        .map(parse_workflow)
        .collect::<HashMap<_, _>>();

    let result = regions
        .next()
        .unwrap()
        .lines()
        .map(parse_parts)
        .filter(|part| run_workflows(*part, &workflows))
        .map(Part::sum)
        .sum::<i64>();

    Some(result)
}

pub fn part_two(input: &str) -> Option<i64> {
    let mut regions = input.split("\n\n");
    let workflows = regions
        .next()
        .unwrap()
        .lines()
        .map(parse_workflow)
        .collect::<HashMap<_, _>>();

    let start_ranges = PartRange {
        x: 1..4001,
        m: 1..4001,
        a: 1..4001,
        s: 1..4001,
    };

    let result = run_ranged_workflows(start_ranges, &workflows);

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result =
            part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(19114));
    }

    #[test]
    fn test_part_two() {
        let result =
            part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(167409079868000));
    }
}
