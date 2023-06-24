// Copyright 2023 Shafish Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use llmchain_common::chat_tokens;
use llmchain_llms::LLM;
use llmchain_loaders::Documents;
use llmchain_prompts::GithubPRSummaryPrompt;
use llmchain_prompts::Prompt;
use llmchain_prompts::PromptTemplate;
use log::info;
use parking_lot::RwLock;

use crate::Summarize;

pub struct GithubPRSummary {
    tokens: RwLock<usize>,
    llm: Arc<dyn LLM>,
    summaries: RwLock<Vec<String>>,
}
impl GithubPRSummary {
    pub fn create(llm: Arc<dyn LLM>) -> Arc<Self> {
        Arc::new(Self {
            tokens: Default::default(),
            llm,
            summaries: RwLock::new(Vec::new()),
        })
    }
}

#[async_trait::async_trait]
impl Summarize for GithubPRSummary {
    async fn add_documents(&self, documents: &Documents) -> Result<()> {
        for (i, document) in documents.iter().enumerate() {
            let template = "
You will act as a reviewer for GitHub Pull Requests.
Please write a understandable key changes summaries on the following git diff, give as bullet points:

```diff
{text}
```
";
            let prompt_template = PromptTemplate::create(template, vec!["text".to_string()]);
            let mut input_variables = HashMap::new();
            input_variables.insert("text", document.content.as_str());
            let prompt = prompt_template.format(input_variables)?;

            let tokens = chat_tokens(&prompt)?;
            *self.tokens.write() += tokens.len();

            let summary = self.llm.generate(&prompt).await?;
            info!(
                "summary [{}/{}, tokens {}]: \n{}",
                i + 1,
                documents.len(),
                tokens.len(),
                summary.generation
            );
            self.summaries.write().push(summary.generation);
        }

        Ok(())
    }

    async fn final_summary(&self) -> Result<String> {
        if self.summaries.read().is_empty() {
            return Ok("".to_string());
        }

        let mut input_variables = HashMap::new();
        let text = self.summaries.read().join("\n");
        input_variables.insert("text", text.as_str());

        let prompt_template = GithubPRSummaryPrompt::create();
        let prompt = prompt_template.format(input_variables)?;

        let tokens = chat_tokens(&prompt)?;
        *self.tokens.write() += tokens.len();
        info!("prompt: tokens {}, result\n{}", tokens.len(), prompt);

        let summary = self.llm.generate(&prompt).await?;
        info!("final summary: {}", summary.generation);

        Ok(summary.generation)
    }

    fn tokens(&self) -> usize {
        *self.tokens.read()
    }
}
