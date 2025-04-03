use crate::rvm_htab::{self, RvmHtabCtx};

const TVM_LEX_MAX_TOKENS: usize = 1024;

pub struct RvmLexerCtx {
    pub tokens: Vec<Vec<String>>
}

impl RvmLexerCtx {
    pub fn new() -> Self {
        RvmLexerCtx {
            tokens: vec![]
        }
    }

    pub fn rvm_lex(&mut self, source: &str, defines: &RvmHtabCtx) {
        // Split the source into individual lines 
        let lines: Vec<String> = source.lines()
                            .map(|line| {
                                /* Ignore comments delimited by '#' */                                
                                if let Some(comment_index) = line.find('#') {
                                    line[..comment_index].trim().to_string()
                                } else {
                                    line.trim().to_string()
                                }
                            })
                            .filter(|line| !line.is_empty()) // Ignore empty lines
                            .collect();

        for line in lines {
            let mut line_toks: Vec<String> = Vec::with_capacity(TVM_LEX_MAX_TOKENS);
            for tok in line.split([' ', '\t', ',']) {
                // Check if token exists in defines map 
                let resolved_tok = defines.rvm_htab_find_ref(tok);
                match resolved_tok {
                    Some(resolved) => {
                        line_toks.push(resolved);
                    }
                    None => {
                        line_toks.push(tok.to_string());
                    }
                }
                if line_toks.len() >= TVM_LEX_MAX_TOKENS {
                    break;
                }
            }
        }
    }
}