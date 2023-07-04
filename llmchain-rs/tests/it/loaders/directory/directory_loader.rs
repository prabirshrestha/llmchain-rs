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

use std::io::Write;

use anyhow::Result;
use goldenfile::Mint;
use llmchain::DirectoryLoader;
use llmchain::DocumentLoader;
use llmchain::DocumentPath;
use llmchain::LocalDisk;
use llmchain::MarkdownLoader;

#[tokio::test]
async fn test_directory_loader() -> Result<()> {
    // testdata dir.
    let curdir = std::env::current_dir()?.to_str().unwrap().to_string();
    let testdata_dir = format!("{}/tests/testdata", curdir);
    let directory_dir = format!("{}/directory/", testdata_dir);

    // Load
    let markdown_loader = MarkdownLoader::create(LocalDisk::create()?);
    let directory_loader =
        DirectoryLoader::create(LocalDisk::create()?).with_loader("**/*.md", markdown_loader);
    let documents = directory_loader
        .load(DocumentPath::from_string(&directory_dir))
        .await?;
    assert_eq!(documents.len(), 2);

    // Check.
    let mut mint = Mint::new(&testdata_dir);
    let golden_path = "directory/directory_loader.golden";
    let mut file = mint.new_goldenfile(golden_path)?;
    for (i, doc) in documents.iter().enumerate() {
        writeln!(
            file,
            "part={}, len={}, md5={}",
            i,
            doc.content.len(),
            doc.content_md5
        )?;
        writeln!(
            file,
            "------------------------------------------------------------"
        )?;
        writeln!(file, "{}", doc.content)?;
    }

    Ok(())
}
