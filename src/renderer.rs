//! Renders a [`Document`] to a complete, standalone LaTeX document targeting
//! Jake Gutierrez's popular resume template.
//!
//! The output is self-contained (`\documentclass` … `\end{document}`) and
//! compiles to a PDF with `pdflatex`. All user-supplied text is passed through
//! [`latex_escape`] so characters like `&`, `%` and `_` cannot break the build.
//!
//! Layout: with no `sidebar`, the document is the classic single-column Jake
//! template. When a `sidebar` is present, it switches to a two-column layout —
//! a narrow left rail (name + contact + sidebar fields) beside the main column
//! (summary + sections). The resume macros use `\linewidth` rather than
//! `\textwidth`, so the same macros render correctly in either context.

use std::fmt::Write;

use crate::ast::{Document, Entry, Field, Section, SectionBody};

/// Jake's resume preamble plus the helper macros the body relies on.
const PREAMBLE: &str = r#"\documentclass[letterpaper,11pt]{article}

\usepackage[empty]{fullpage}
\usepackage{titlesec}
\usepackage[usenames,dvipsnames]{color}
\usepackage{enumitem}
\usepackage[hidelinks]{hyperref}
\usepackage{fancyhdr}
\usepackage[english]{babel}
\usepackage{tabularx}

\pagestyle{fancy}
\fancyhf{}
\fancyfoot{}
\renewcommand{\headrulewidth}{0pt}
\renewcommand{\footrulewidth}{0pt}

\addtolength{\oddsidemargin}{-0.5in}
\addtolength{\evensidemargin}{-0.5in}
\addtolength{\textwidth}{1in}
\addtolength{\topmargin}{-.5in}
\addtolength{\textheight}{1.0in}

\urlstyle{same}

\raggedbottom
\raggedright
\setlength{\tabcolsep}{0in}

% Section heading formatting
\titleformat{\section}{
  \vspace{-4pt}\scshape\raggedright\large
}{}{0em}{}[\color{black}\titlerule \vspace{-5pt}]

% Resume helper macros (width-relative so they work in either column)
\newcommand{\resumeItem}[1]{
  \item\small{
    {#1 \vspace{-2pt}}
  }
}

\newcommand{\resumeSubheading}[4]{
  \vspace{-2pt}\item
    \begin{tabular*}{0.97\linewidth}[t]{l@{\extracolsep{\fill}}r}
      \textbf{#1} & #2 \\
      \textit{\small#3} & \textit{\small #4} \\
    \end{tabular*}\vspace{-7pt}
}

\newcommand{\resumeSubSubheading}[2]{
    \item
    \begin{tabular*}{0.97\linewidth}{l@{\extracolsep{\fill}}r}
      \textit{\small#1} & \textit{\small #2} \\
    \end{tabular*}\vspace{-7pt}
}

\renewcommand\labelitemii{$\vcenter{\hbox{\tiny$\bullet$}}$}

\newcommand{\resumeSubHeadingListStart}{\begin{itemize}[leftmargin=0.15in, label={}]}
\newcommand{\resumeSubHeadingListEnd}{\end{itemize}}
\newcommand{\resumeItemListStart}{\begin{itemize}}
\newcommand{\resumeItemListEnd}{\end{itemize}\vspace{-5pt}}
"#;

/// Render `doc` to a full LaTeX document string.
pub fn render(doc: &Document) -> String {
    let mut out = String::new();
    out.push_str(PREAMBLE);
    out.push_str("\n\\begin{document}\n\n");

    if doc.sidebar.is_empty() {
        // Classic single-column Jake.
        render_centered_header(doc, &mut out);
        render_main_content(doc, &mut out);
    } else {
        // Two-column: sidebar rail + main content.
        render_two_column(doc, &mut out);
    }

    out.push_str("\n\\end{document}\n");
    out
}

/// Summary (if any) followed by the sections — the body shared by both layouts.
fn render_main_content(doc: &Document, out: &mut String) {
    if !doc.summary.is_empty() {
        render_summary(&doc.summary, out);
    }
    for section in &doc.sections {
        render_section(section, out);
    }
}

/// Single-column header: centered name + a `$|$`-separated contact line.
fn render_centered_header(doc: &Document, out: &mut String) {
    out.push_str("\\begin{center}\n");
    if let Some(name) = &doc.name {
        let _ = writeln!(
            out,
            "    \\textbf{{\\Huge \\scshape {}}} \\\\ \\vspace{{1pt}}",
            latex_escape(name)
        );
    }

    let parts: Vec<String> = doc.contact.iter().map(render_contact_field).collect();
    if !parts.is_empty() {
        out.push_str("    \\small ");
        out.push_str(&parts.join(" $|$ "));
        out.push('\n');
    }
    out.push_str("\\end{center}\n\n");
}

/// Two-column layout. The sidebar rail holds the name, contact, and sidebar
/// fields; the main column holds the summary and sections. Page-breaking is a
/// non-issue for the one-page resumes this targets, so simple `minipage`s keep
/// the LaTeX robust.
fn render_two_column(doc: &Document, out: &mut String) {
    out.push_str("\\noindent\n");
    out.push_str("\\begin{minipage}[t]{0.30\\textwidth}\n\\raggedright\n");
    render_sidebar_column(doc, out);
    out.push_str("\n\\end{minipage}%\n\\hfill\n");
    out.push_str("\\begin{minipage}[t]{0.66\\textwidth}\n");
    render_main_content(doc, out);
    out.push_str("\\end{minipage}\n");
}

/// The left rail: name, then contact + sidebar fields stacked one per line.
fn render_sidebar_column(doc: &Document, out: &mut String) {
    if let Some(name) = &doc.name {
        let _ = writeln!(out, "{{\\Huge \\scshape {}}}\\\\[8pt]", latex_escape(name));
    }
    let parts: Vec<String> = doc
        .contact
        .iter()
        .chain(doc.sidebar.iter())
        .map(render_contact_field)
        .collect();
    if !parts.is_empty() {
        out.push_str("\\small\n");
        out.push_str(&parts.join("\\\\[4pt]\n"));
    }
}

/// Turn a contact/sidebar field into a LaTeX snippet, linkifying the keys that
/// represent URLs and leaving everything else as escaped plain text.
fn render_contact_field(field: &Field) -> String {
    let value = field.value.trim();
    match field.key.as_str() {
        "email" => format!(
            "\\href{{mailto:{}}}{{\\underline{{{}}}}}",
            url_escape(value),
            latex_escape(value)
        ),
        "github" => hyperlink(value, "https://github.com/"),
        "gitlab" => hyperlink(value, "https://gitlab.com/"),
        "linkedin" => hyperlink(value, "https://linkedin.com/in/"),
        "twitter" | "x" => hyperlink(value, "https://x.com/"),
        "orcid" => hyperlink(value, "https://orcid.org/"),
        "scholar" | "website" | "site" | "link" | "url" => hyperlink(value, "https://"),
        // location, phone, languages, skills and anything else: plain text.
        _ => latex_escape(value),
    }
}

/// Build an `\href{url}{\underline{display}}`, normalising bare handles into a
/// full URL using `prefix` while keeping the visible text human-friendly.
fn hyperlink(value: &str, prefix: &str) -> String {
    let url = if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else if value.contains('.') || value.contains('/') {
        format!("https://{value}")
    } else {
        format!("{prefix}{value}")
    };
    let display = value
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    format!(
        "\\href{{{}}}{{\\underline{{{}}}}}",
        url_escape(&url),
        latex_escape(display)
    )
}

fn render_summary(summary: &[String], out: &mut String) {
    out.push_str("\\section{Summary}\n");
    out.push_str("\\resumeItemListStart\n");
    for item in summary {
        let _ = writeln!(out, "  \\resumeItem{{{}}}", latex_escape(item));
    }
    out.push_str("\\resumeItemListEnd\n\n");
}

fn render_section(section: &Section, out: &mut String) {
    let _ = writeln!(out, "\\section{{{}}}", latex_escape(&section.title));
    match &section.body {
        SectionBody::Entries(entries) => {
            // An empty `itemize` is a LaTeX error ("missing \item"), so only
            // open the list when there is at least one entry.
            if entries.is_empty() {
                out.push('\n');
            } else {
                out.push_str("  \\resumeSubHeadingListStart\n");
                for entry in entries {
                    render_entry(entry, out);
                }
                out.push_str("  \\resumeSubHeadingListEnd\n\n");
            }
        }
        SectionBody::Tags(tags) => {
            out.push_str("  \\begin{itemize}[leftmargin=0.15in, label={}]\n");
            let _ = writeln!(out, "    \\small{{\\item{{{}}}}}", latex_escape(tags));
            out.push_str("  \\end{itemize}\n\n");
        }
    }
}

fn render_entry(entry: &Entry, out: &mut String) {
    let blank = String::new();
    let role = entry.role.as_ref().unwrap_or(&blank);
    let when = entry.when.as_ref().unwrap_or(&blank);
    let org = entry.org.as_ref().unwrap_or(&blank);
    let location = entry.location.as_ref().unwrap_or(&blank);

    // The role doubles as a link target when a `link` is supplied.
    let role_tex = match &entry.link {
        Some(link) if !link.is_empty() => format!(
            "\\href{{{}}}{{\\underline{{{}}}}}",
            url_escape(link),
            latex_escape(role)
        ),
        _ => latex_escape(role),
    };

    let _ = writeln!(
        out,
        "    \\resumeSubheading\n      {{{}}}{{{}}}\n      {{{}}}{{{}}}",
        role_tex,
        latex_escape(when),
        latex_escape(org),
        latex_escape(location),
    );

    let has_items = entry.stack.is_some() || !entry.bullets.is_empty();
    if has_items {
        out.push_str("      \\resumeItemListStart\n");
        if let Some(stack) = &entry.stack {
            let _ = writeln!(
                out,
                "        \\resumeItem{{\\textit{{Stack:}} {}}}",
                latex_escape(stack)
            );
        }
        for bullet in &entry.bullets {
            let _ = writeln!(out, "        \\resumeItem{{{}}}", latex_escape(bullet));
        }
        out.push_str("      \\resumeItemListEnd\n");
    }
}

/// Escape characters that are special in LaTeX body text.
///
/// Done character-by-character so escape sequences we emit (which themselves
/// contain backslashes) are never re-escaped.
pub fn latex_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("\\&"),
            '%' => out.push_str("\\%"),
            '$' => out.push_str("\\$"),
            '#' => out.push_str("\\#"),
            '_' => out.push_str("\\_"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            '\\' => out.push_str("\\textbackslash{}"),
            _ => out.push(c),
        }
    }
    out
}

/// Minimal escaping for the URL argument of `\href` (only the characters that
/// would otherwise terminate or break the argument).
fn url_escape(s: &str) -> String {
    s.replace('%', "\\%").replace('#', "\\#")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Field;

    fn doc_with_one_entry() -> Document {
        Document {
            name: Some("R&D Person".into()),
            contact: vec![Field {
                key: "email".into(),
                value: "a@b.com".into(),
            }],
            summary: vec![],
            sidebar: vec![],
            sections: vec![Section {
                title: "Experience".into(),
                body: SectionBody::Entries(vec![Entry {
                    role: Some("Engineer".into()),
                    org: Some("Acme".into()),
                    when: Some("2025".into()),
                    bullets: vec!["Did 50% more".into()],
                    ..Entry::default()
                }]),
            }],
        }
    }

    #[test]
    fn emits_standalone_document() {
        let tex = render(&doc_with_one_entry());
        assert!(tex.contains("\\documentclass[letterpaper,11pt]{article}"));
        assert!(tex.contains("\\begin{document}"));
        assert!(tex.contains("\\end{document}"));
        assert!(tex.contains("\\resumeSubheading"));
        assert!(tex.contains("Engineer"));
        assert!(tex.contains("Did 50\\% more"));
    }

    #[test]
    fn single_column_when_no_sidebar() {
        let tex = render(&doc_with_one_entry());
        assert!(tex.contains("\\begin{center}"));
        assert!(!tex.contains("\\begin{minipage}"));
    }

    #[test]
    fn two_column_when_sidebar_present() {
        let mut doc = doc_with_one_entry();
        doc.sidebar = vec![Field {
            key: "languages".into(),
            value: "English, French".into(),
        }];
        let tex = render(&doc);
        assert!(tex.contains("\\begin{minipage}[t]{0.30\\textwidth}"));
        assert!(tex.contains("\\begin{minipage}[t]{0.66\\textwidth}"));
        assert!(tex.contains("English, French"));
        // Name appears in the sidebar rail, not the centered header.
        assert!(!tex.contains("\\begin{center}"));
    }

    #[test]
    fn escapes_special_characters() {
        assert_eq!(latex_escape("R&D 100% _x_"), "R\\&D 100\\% \\_x\\_");
        let tex = render(&doc_with_one_entry());
        assert!(tex.contains("R\\&D Person"));
    }

    #[test]
    fn empty_section_emits_no_itemize() {
        // An empty entries list must not produce an empty `itemize` (LaTeX error).
        let doc = Document {
            sections: vec![Section {
                title: "Empty".into(),
                body: SectionBody::Entries(vec![]),
            }],
            ..Document::default()
        };
        let tex = render(&doc);
        assert!(tex.contains("\\section{Empty}"));
        // The list macro is *defined* in the preamble; assert it is not *invoked*
        // in the body (invocations are indented two spaces).
        assert!(!tex.contains("  \\resumeSubHeadingListStart"));
    }

    #[test]
    fn renders_tags_section() {
        let doc = Document {
            sections: vec![Section {
                title: "Skills".into(),
                body: SectionBody::Tags("Rust, Python".into()),
            }],
            ..Document::default()
        };
        let tex = render(&doc);
        assert!(tex.contains("\\section{Skills}"));
        assert!(tex.contains("Rust, Python"));
    }

    #[test]
    fn github_handle_becomes_full_url() {
        let field = Field {
            key: "github".into(),
            value: "octocat".into(),
        };
        let snippet = render_contact_field(&field);
        assert!(snippet.contains("https://github.com/octocat"));
        assert!(snippet.contains("\\underline{octocat}"));
    }

    #[test]
    fn richer_contact_keys() {
        let orcid = render_contact_field(&Field {
            key: "orcid".into(),
            value: "0000-0002-1825-0097".into(),
        });
        assert!(orcid.contains("https://orcid.org/0000-0002-1825-0097"));

        let x = render_contact_field(&Field {
            key: "x".into(),
            value: "jack".into(),
        });
        assert!(x.contains("https://x.com/jack"));

        // Phone is plain (escaped) text, not a link.
        let phone = render_contact_field(&Field {
            key: "phone".into(),
            value: "+1 555 010".into(),
        });
        assert!(!phone.contains("\\href"));
        assert!(phone.contains("+1 555 010"));
    }
}
