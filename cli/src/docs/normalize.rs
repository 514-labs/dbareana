use scraper::{ElementRef, Html, Selector};

#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub section_path: String,
    pub body: String,
}

pub fn normalize_html(html: &str) -> Vec<Section> {
    let document = Html::parse_document(html);
    let heading_selector = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
    let title_selector = Selector::parse("title").unwrap();

    let default_title = document
        .select(&title_selector)
        .next()
        .map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "Document".to_string());

    let headings: Vec<ElementRef> = document.select(&heading_selector).collect();
    if headings.is_empty() {
        let text = html2text::from_read(html.as_bytes(), 120).trim().to_string();
        return vec![Section {
            title: default_title.clone(),
            section_path: default_title,
            body: text,
        }];
    }

    let mut sections = Vec::new();
    let mut stack: Vec<String> = Vec::new();

    for heading in headings {
        let title = heading.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if title.is_empty() {
            continue;
        }
        let level = heading
            .value()
            .name()
            .trim_start_matches('h')
            .parse::<usize>()
            .unwrap_or(1);
        while stack.len() >= level {
            stack.pop();
        }
        stack.push(title.clone());
        let section_path = stack.join(" → ");

        let mut body = String::new();
        let mut node = heading.next_sibling();
        while let Some(sib) = node {
            if let Some(el) = ElementRef::wrap(sib.clone()) {
                if heading_selector.matches(&el) {
                    break;
                }
                let text = el.text().collect::<Vec<_>>().join(" ");
                if !text.trim().is_empty() {
                    body.push_str(text.trim());
                    body.push('\n');
                }
            }
            node = sib.next_sibling();
        }

        let body = body.trim().to_string();
        if body.is_empty() {
            continue;
        }

        sections.push(Section {
            title,
            section_path,
            body,
        });
    }

    if sections.is_empty() {
        let text = html2text::from_read(html.as_bytes(), 120).trim().to_string();
        return vec![Section {
            title: default_title.clone(),
            section_path: default_title,
            body: text,
        }];
    }

    sections
}

pub fn normalize_markdown(markdown: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut current_title = "Document".to_string();
    let mut current_body = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            if !current_body.trim().is_empty() {
                let section_path = if stack.is_empty() {
                    current_title.clone()
                } else {
                    stack.join(" → ")
                };
                sections.push(Section {
                    title: current_title.clone(),
                    section_path,
                    body: current_body.trim().to_string(),
                });
                current_body.clear();
            }
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            let title = trimmed.trim_start_matches('#').trim().to_string();
            if title.is_empty() {
                continue;
            }
            while stack.len() >= level {
                stack.pop();
            }
            stack.push(title.clone());
            current_title = title;
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    if !current_body.trim().is_empty() {
        let section_path = if stack.is_empty() {
            current_title.clone()
        } else {
            stack.join(" → ")
        };
        sections.push(Section {
            title: current_title,
            section_path,
            body: current_body.trim().to_string(),
        });
    }

    if sections.is_empty() {
        sections.push(Section {
            title: "Document".to_string(),
            section_path: "Document".to_string(),
            body: markdown.trim().to_string(),
        });
    }

    sections
}

pub fn normalize_info(info: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    for node in info.split('\u{1f}') {
        let trimmed = node.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut lines = trimmed.lines();
        let mut title = "Node".to_string();
        let mut body_lines = Vec::new();
        let mut header_done = false;
        while let Some(line) = lines.next() {
            if !header_done && line.trim().is_empty() {
                header_done = true;
                continue;
            }
            if !header_done {
                if let Some(idx) = line.find("Node:") {
                    let node_name = line[idx + 5..]
                        .split(',')
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !node_name.is_empty() {
                        title = node_name.to_string();
                    }
                }
            } else {
                body_lines.push(line);
            }
        }

        let body = body_lines.join("\n").trim().to_string();
        if body.is_empty() {
            continue;
        }
        sections.push(Section {
            title: title.clone(),
            section_path: title,
            body,
        });
    }

    if sections.is_empty() && !info.trim().is_empty() {
        sections.push(Section {
            title: "Document".to_string(),
            section_path: "Document".to_string(),
            body: info.trim().to_string(),
        });
    }

    sections
}
