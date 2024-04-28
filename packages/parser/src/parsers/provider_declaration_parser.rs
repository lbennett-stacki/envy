use crate::{
    abstract_syntax_tree::{AbstractSyntaxNode, DeclarationNode, Leaf},
    parsers::attribute_block_parser::AttributeBlockParser,
    providers::{PartialProviderDeclarationNode, ProviderDeclarationNode},
    Parser,
};
use nv_lexer::{
    tokens::{LexerSymbol, LexerToken},
    LexerTokenKind,
};
use std::sync::Weak;

pub struct ProviderDeclarationParser;

impl Parser<Option<AbstractSyntaxNode>> for ProviderDeclarationParser {
    fn parse(
        tokens: &[LexerToken],
        parent: Weak<AbstractSyntaxNode>,
    ) -> (usize, Option<AbstractSyntaxNode>) {
        let mut tokens_iter = tokens.iter().enumerate();
        let mut buffer = vec![];
        let mut ast_fragment = None;

        let mut processed_count = 0;

        let mut partial_declaration = PartialProviderDeclarationNode {
            identifier: None,
            type_value: None,
            attributes: vec![],
        };

        while let Some((index, token)) = tokens_iter.next() {
            let token = token.to_owned();

            processed_count += 1;

            let sub_tokens = &tokens[index..].to_vec();
            let sub_tokens = sub_tokens.to_vec();

            match token.kind {
                LexerTokenKind::Identifier(identifier) => {
                    partial_declaration.identifier = Some(Leaf::new(identifier, token.range));

                    continue;
                }
                LexerTokenKind::ProviderType(type_value) => {
                    partial_declaration.type_value = Some(Leaf::new(type_value, token.range));

                    continue;
                }
                LexerTokenKind::Symbol(LexerSymbol::BlockOpenCurly) => {
                    let (count, parsed_block) =
                        AttributeBlockParser::parse(&sub_tokens, parent.clone());

                    // -1 because we dont want to double count the block open curly
                    let count = count - 1;

                    partial_declaration.attributes = parsed_block
                        .iter()
                        .map(|declaration| declaration.clone().into())
                        .collect();

                    processed_count += count;
                    if count > 0 {
                        tokens_iter.nth(count - 1);
                    }

                    let declaration: Result<ProviderDeclarationNode, _> =
                        partial_declaration.clone().try_into();

                    if declaration.is_ok() {
                        ast_fragment = Some(AbstractSyntaxNode::Declaration(
                            DeclarationNode::ProviderDeclaration(declaration.unwrap()),
                        ));
                        return (processed_count, ast_fragment);
                    }

                    continue;
                }
                LexerTokenKind::Symbol(LexerSymbol::Newline) => {
                    let declaration: Result<ProviderDeclarationNode, _> =
                        partial_declaration.clone().try_into();

                    if declaration.is_ok() {
                        ast_fragment = Some(AbstractSyntaxNode::Declaration(
                            DeclarationNode::ProviderDeclaration(declaration.unwrap()),
                        ));

                        return (processed_count, ast_fragment);
                    }

                    continue;
                }
                _ => {
                    buffer.push(token);
                    continue;
                }
            };
        }

        let declaration: Result<ProviderDeclarationNode, _> =
            partial_declaration.clone().try_into();

        if declaration.is_ok() {
            ast_fragment = Some(AbstractSyntaxNode::Declaration(
                DeclarationNode::ProviderDeclaration(declaration.unwrap()),
            ));

            return (processed_count, ast_fragment);
        }

        (processed_count, ast_fragment)
    }
}
