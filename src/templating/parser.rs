// https://github.com/fflorent/nom_locate/ Line numbers?

use super::{
    directive::{DirectiveBlock, DoNothing},
    template::{Template, TemplateBlock, TemplateDirectiveBlock},
};
use anyhow::Result;
use std::path::PathBuf;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till, take_until},
    character::complete::char,
    combinator::{map, not},
    multi::many0,
    sequence::{delimited, pair, terminated},
    IResult,
};

// Parses a raw template string into a Template
pub(super) fn parse_template(raw_template: &str) -> Result<Template> {
    match template(raw_template) {
        Ok((_, blocks)) => Ok(Template { blocks }),
        Err(e) => anyhow::bail!("{}", e), // Rethrow the error (lifetimes stuff)
    }
}

// PARSER CODE

const OPENING_MARK: &str = "!!%";
const CLOSING_MARK: &str = "%!!";

/*
 * text    TemplateBlock::Text
 * ( ... ) TemplateBlock::BlockDirective
 * text    TemplateBlock::Text
 * ( ... ) TemplateBlock::BlockDirective
 * text    TemplateBlock::Text
 */
fn template(input: &str) -> IResult<&str, Vec<TemplateBlock>> {
    many0(alt((
        map(directive_block, TemplateBlock::BlockDirective),
        map(text, |t| TemplateBlock::Text(t.to_string())),
    )))(input)
}

fn text(input: &str) -> IResult<&str, &str> {
    map(is_not(OPENING_MARK), |t: &str| t.trim())(input)
}

/*
 * text
 * directive_block
 */
fn template_block(input: &str) -> IResult<&str, TemplateBlock> {
    alt((
        map(directive_block, TemplateBlock::BlockDirective),
        map(is_not(CLOSING_MARK), |t: &str| {
            TemplateBlock::Text(t.trim().to_string())
        }),
    ))(input)
}

/*
 * ( directive template_blocks )
 */
fn directive_block(input: &str) -> IResult<&str, TemplateDirectiveBlock> {
    let (rest, (directive, blocks)) = delimited(
        tag(OPENING_MARK),
        pair(block_directive, many0(template_block)),
        tag(CLOSING_MARK),
    )(input)?;

    Ok((rest, TemplateDirectiveBlock { directive, blocks }))
}

fn block_directive(input: &str) -> IResult<&str, Box<dyn DirectiveBlock>> {
    let (rest, parsed) = terminated(map(is_not("\n"), |t: &str| t.trim()), char('\n'))(input)?;

    // Because we cant pass this as a reference, we will need Clone later
    let directive = Box::new(DoNothing {
        text: parsed.to_string(),
    });

    Ok((rest, directive))
}

// NOTE: Test no work because of Eq / PartialEq
#[cfg(test)]
mod tests {
    use std::fmt::format;

    use super::*;

    fn compare_vec_template_blocks(v1: &Vec<TemplateBlock>, v2: &Vec<TemplateBlock>) -> bool {
        assert_eq!(v1.len(), v2.len());
        for (b1, b2) in v1.iter().zip(v2.iter()) {
            if compare_template_blocks(b1, b2) == false {
                return false;
            }
        }
        true
    }

    fn compare_template_blocks(t1: &TemplateBlock, t2: &TemplateBlock) -> bool {
        match (t1, t2) {
            (TemplateBlock::Text(t1), TemplateBlock::Text(t2)) => t1 == t2,
            (TemplateBlock::BlockDirective(t1), TemplateBlock::BlockDirective(t2)) => {
                format!("{:?}", t1.directive).cmp(&format!("{:?}", t2.directive))
                    == std::cmp::Ordering::Equal
                    && t1
                        .blocks
                        .iter()
                        .zip(t2.blocks.iter())
                        .all(|(t1, t2)| compare_template_blocks(t1, t2))
            }
            (TemplateBlock::LineDirective(t1), TemplateBlock::LineDirective(t2)) => {
                format!("{:?}", t1.directive).cmp(&format!("{:?}", t2.directive))
                    == std::cmp::Ordering::Equal
            }
            _ => false,
        }
    }

    #[test]
    fn test_template() {
        let input = format!(
            r#"
 textbefore
 {} directive1
   text1
 {}
 textbetween
 {} directive2
   text2
 {}
 textafter
 "#,
            OPENING_MARK, CLOSING_MARK, OPENING_MARK, CLOSING_MARK
        );
        let expected = vec![
            TemplateBlock::Text("textbefore".to_string()),
            TemplateBlock::BlockDirective(TemplateDirectiveBlock {
                directive: Box::new(DoNothing {
                    text: "directive1".to_string(),
                }),
                blocks: vec![TemplateBlock::Text("text1".to_string())],
            }),
            TemplateBlock::Text("textbetween".to_string()),
            TemplateBlock::BlockDirective(TemplateDirectiveBlock {
                directive: Box::new(DoNothing {
                    text: "directive2".to_string(),
                }),
                blocks: vec![TemplateBlock::Text("text2".to_string())],
            }),
            TemplateBlock::Text("textafter".to_string()),
        ];

        let result = template(input.as_str()).unwrap().1;
        assert!(compare_vec_template_blocks(&result, &expected));
    }

    #[test]
    fn test_template_block() {
        let input = format!(
            "{} directive1 \n{} directive2 \ntext{} asd{}",
            OPENING_MARK, OPENING_MARK, CLOSING_MARK, CLOSING_MARK
        );
        let wrong_input1 = format!(
            "{} directive1 \n{} directive2 \ntext{} asd{}",
            OPENING_MARK, "?", CLOSING_MARK, CLOSING_MARK
        );
        let wrong_input2 = format!(
            "{} directive1 \n{} directive2 \ntext{} asd{}",
            OPENING_MARK, "?", CLOSING_MARK, CLOSING_MARK
        );
        let expected = TemplateBlock::BlockDirective(TemplateDirectiveBlock {
            directive: Box::new(DoNothing {
                text: "directive1".to_string(),
            }),
            blocks: vec![
                TemplateBlock::BlockDirective(TemplateDirectiveBlock {
                    directive: Box::new(DoNothing {
                        text: "directive2".to_string(),
                    }),
                    blocks: vec![TemplateBlock::Text("text".to_string())],
                }),
                TemplateBlock::Text("asd".to_string()),
            ],
        });

        let result = template_block(input.as_str()).unwrap().1;
        let wrong_result1 = template_block(wrong_input1.as_str()).unwrap().1;
        let wrong_result2 = template_block(wrong_input2.as_str()).unwrap().1;
        assert!(false == compare_template_blocks(&wrong_result1, &expected));
        assert!(false == compare_template_blocks(&wrong_result2, &expected));
        assert!(compare_template_blocks(&result, &expected));
    }
}
