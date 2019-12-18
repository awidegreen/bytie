use nom::bytes::complete::take;
use nom::character::complete::{alpha1, digit1};
use nom::combinator::{map_opt, map_res, opt};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq, Eq)]
enum ActionType {
    Add,
    Delete,
    Replace,
}

#[derive(Debug, PartialEq, Eq)]
struct Action {
    action_type: ActionType,
    begin: usize,
    bhvr: Option<Behavior>,
    end: Option<usize>,
}

impl ActionType {
    fn from(input: &str) -> Result<ActionType, ()> {
        match input {
            "d" | "del" | "delete" => Ok(ActionType::Delete),
            "a" | "add" => Ok(ActionType::Add),
            "r" | "replace" | "rpl" => Ok(ActionType::Replace),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Behavior {
    Until,
    Length,
}

impl Behavior {
    fn from(input: &str) -> Result<Behavior, ()> {
        match input {
            ":" => Ok(Behavior::Until),
            "+" => Ok(Behavior::Length),
            _ => Err(()),
        }
    }
}

fn number(input: &str) -> IResult<&str, usize> {
    map_opt(digit1, |s: &str| s.parse::<usize>().ok())(input)
}

fn behavior(input: &str) -> IResult<&str, Behavior> {
    map_res(take(1usize), Behavior::from)(input)
}

fn action_type(input: &str) -> IResult<&str, ActionType> {
    map_res(alpha1, ActionType::from)(input)
}

fn action(input: &str) -> IResult<&str, Action> {
    let (input, action_type) = action_type(input)?;
    let (input, (begin, bhvr, end)) = tuple((number, opt(behavior), opt(number)))(input)?;

    Ok((
        input,
        Action {
            action_type,
            begin,
            bhvr,
            end,
        },
    ))
}

#[derive(Debug, PartialEq, Eq)]
pub struct DeleteAction {
    pub begin: usize,
    pub behavior: Option<Behavior>,
    pub end: Option<usize>,
}

pub fn del_action(input: &str) -> IResult<&str, DeleteAction> {
    let (input, (begin, beh, end)) = tuple((number, opt(behavior), opt(number)))(input)?;

    Ok((
        input,
        DeleteAction {
            begin,
            behavior: beh,
            end: end,
        },
    ))
}

#[test]
fn test_number() {
    assert_eq!(number("123"), Ok(("", 123)));
    assert_eq!(number("123 a"), Ok((" a", 123)));

    assert_eq!(
        number("abc"),
        Err(nom::Err::Error(("abc", nom::error::ErrorKind::Digit)))
    );
}

#[test]
fn test_behavior() {
    assert_eq!(behavior(":"), Ok(("", Behavior::Until)));
    assert_eq!(behavior("+123"), Ok(("123", Behavior::Length)));
    assert_eq!(behavior(":1"), Ok(("1", Behavior::Until)));
    assert_eq!(behavior("++"), Ok(("+", Behavior::Length)));

    assert_eq!(
        behavior("xx"),
        Err(nom::Err::Error(("xx", nom::error::ErrorKind::MapRes)))
    );
}

#[test]
fn test_action_type() {
    assert_eq!(action_type("d"), Ok(("", ActionType::Delete)));
    assert_eq!(action_type("d123"), Ok(("123", ActionType::Delete)));
    assert_eq!(action_type("r1"), Ok(("1", ActionType::Replace)));

    assert_eq!(action_type("add1"), Ok(("1", ActionType::Add)));
    assert_eq!(action_type("del1"), Ok(("1", ActionType::Delete)));

    assert_eq!(
        action_type("aa"),
        Err(nom::Err::Error(("aa", nom::error::ErrorKind::MapRes)))
    );
    assert_eq!(
        action_type("xx"),
        Err(nom::Err::Error(("xx", nom::error::ErrorKind::MapRes)))
    );
}

#[test]
fn test_action() {
    assert_eq!(
        action("d0:10"),
        Ok((
            "",
            Action {
                action_type: ActionType::Delete,
                begin: 0,
                bhvr: Some(Behavior::Until),
                end: Some(10)
            }
        ))
    );

    assert_eq!(
        action("d0"),
        Ok((
            "",
            Action {
                action_type: ActionType::Delete,
                begin: 0,
                bhvr: None,
                end: None
            }
        ))
    );

    assert_eq!(
        action("replace0"),
        Ok((
            "",
            Action {
                action_type: ActionType::Replace,
                begin: 0,
                bhvr: None,
                end: None
            }
        ))
    );
}
