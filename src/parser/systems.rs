use pest::iterators::Pairs;

use crate::ast;

use super::*;

/// Parses a system.
pub fn parse_system<'a>(mut pairs: Pairs<'a, Rule>) -> (&'a str, ast::Name<'a>) {
    let name = pairs.next().unwrap().as_str();
    let filters = parse_filter_list(pairs.next().unwrap().into_inner());
    let code = parse_block(pairs.next().unwrap().into_inner());

    let system = ast::System {
        filters,
        code,
    };

    (name, ast::Name::System(system))
}

/// Parses a list of filters.
pub fn parse_filter_list<'a>(mut pairs: Pairs<'a, Rule>) -> Vec<ast::Filter> {
    let mut filters = Vec::new();

    for pair in pairs {
        filters.push(parse_filter(pair.into_inner()));
    }

    filters
}

/// Parses a single filter.
pub fn parse_filter<'a>(mut pairs: Pairs<'a, Rule>) -> ast::Filter {
    let pair = pairs.next().unwrap();

    match pair.as_rule() {
        Rule::entity_filter => parse_entity_filter(pair.into_inner()),
        Rule::resource_filter => ast::Filter::Resource(
            parse_argument(pair.into_inner())
        ),
        _ => unreachable!(),
    }
}

/// Parses an entity filter.
pub fn parse_entity_filter<'a>(mut pairs: Pairs<'a, Rule>) -> ast::Filter {
    let name = pairs.next().unwrap().as_str();
    let mut args = Vec::new();

    for pair in pairs {
        args.push(parse_argument(pair.into_inner()));
    }

    ast::Filter::Entity(name, args)
}

/// Parses a formal argument to a function or system call.
pub fn parse_argument<'a>(mut pairs: Pairs<'a, Rule>) -> ast::Argument<'a> {
    let mut pair = pairs.next().unwrap();

    let is_const = matches!(pair.as_rule(), Rule::const_);
    if !is_const {
        pair = pairs.next().unwrap();
    }
    let ty = parse_type(pair.into_inner());
    let name = pairs.next().unwrap().as_str();

    ast::Argument {is_const, ty, name}
}