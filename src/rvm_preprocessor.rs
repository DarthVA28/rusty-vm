use std::{fs, path::Path};

use crate::rvm_htab::RvmHtabCtx;

const TOK_INCLUDE : &str = "%include";
const TOK_DEFINE : &str = "%define";

pub struct RvmPreprocessor {
    pub defines : RvmHtabCtx
}

impl RvmPreprocessor {
    pub fn new() -> Self {
        RvmPreprocessor {
            defines : RvmHtabCtx::new()
        }
    }

    pub fn rvm_preprocess(&mut self, src: &mut String) -> i32 {
        let mut ret: i32 = 1;
        while ret > 0 {
            ret = 0;
            match self.process_includes(src) {
                Ok(res) => {
                    // convert boolean to int
                    ret += res as i32;
                }
                Err(e) => {
                    eprintln!("Error processing includes: {}", e);
                    break;
                }
            }
            match self.process_defines(src) {
                Ok(res) => {
                    ret += res as i32;
                }
                Err(e) => {
                    eprintln!("Error processing defines: {}", e);
                    return -1;
                }
            }
            // Keep going till no includes or defines need to be replaced 
            if ret == 0 {
                break;
            }
        }
        0
    }

    fn process_includes(&mut self, src: &mut String) -> Result<bool, String> {
        if let Some(start) = src.find(TOK_INCLUDE) {
            // Find the end of the line
            let end = src[start..].find('\n').map(|e| start + e).unwrap_or(src.len());
    
            // Extract the filename
            let include_line = &src[start..end];
            let filename = include_line[TOK_INCLUDE.len()..].trim();
    
            // Check if the file exists
            let filepath = Path::new(filename).with_extension("vm");
            if !filepath.exists() {
                return Err(format!("Unable to open file: {}", filename));
            }
    
            // Read file content
            let file_contents: String = fs::read_to_string(filepath)
                .map_err(|_| format!("Failed to read file: {}", filename))?;
    
            src.replace_range(start..end, &file_contents);
            
            return Ok(true);    
        }
        return Ok(false);
    }

    fn process_defines(&mut self, src: &mut String) -> Result<bool, String> {
        // Find the first occurrence of "#define"
        if let Some(start) = src.find(TOK_DEFINE) {
            // Find the end of the line
            let end = src[start..].find('\n').map(|e| start + e).unwrap_or(src.len());

            // Extract the define line
            let define_line = &src[start + TOK_DEFINE.len()..end].trim();

            // Ensure the define statement is not empty
            if define_line.is_empty() {
                return Err("Define missing arguments.".to_string());
            }

            // Split key and value
            let mut parts = define_line.splitn(2, ' ');
            let key = parts.next().unwrap().trim().to_string();
            let value = parts.next().unwrap_or("").trim().to_string();

            // Check for duplicate definitions
            if self.defines.rvm_htab_find(&key).is_some() {
                return Err(format!("Multiple definitions for {}", key));
            }

            // Insert into hash table
            self.defines.rvm_htab_add(&key, 0, &value);

            // Remove the `#define` line from source
            src.replace_range(start..end + 1, ""); 

            return Ok(true);
        }
        Ok(false)
    }
}