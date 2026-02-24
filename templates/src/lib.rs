use leptos::either::Either;
use leptos::prelude::*;

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn date_range_form(action: &str, start: &str, end: &str) -> String {
    format!(
        r#"<form method="GET" action="{action}" style="display:inline-flex;gap:8px;align-items:center">
<select onchange="(function(sel){{
  var f=sel.form,s=f.start,e=f.end,v=sel.value;
  if(v==='custom')return;
  var now=new Date(),y=now.getFullYear(),m=now.getMonth(),d=now.getDate();
  function fmt(dt){{var mm=''+(dt.getMonth()+1),dd=''+dt.getDate();if(mm.length<2)mm='0'+mm;if(dd.length<2)dd='0'+dd;return dt.getFullYear()+'-'+mm+'-'+dd;}}
  var sd,ed=fmt(now);
  if(v==='7d'){{sd=fmt(new Date(y,m,d-6));}}
  else if(v==='30d'){{sd=fmt(new Date(y,m,d-29));}}
  else if(v==='cm'){{sd=fmt(new Date(y,m,1));}}
  else if(v==='lm'){{sd=fmt(new Date(y,m-1,1));ed=fmt(new Date(y,m,0));}}
  else if(v==='3m'){{sd=fmt(new Date(y,m-2,1));}}
  s.value=sd;e.value=ed;
}})(this)">
<option value="custom">Custom</option>
<option value="7d">Past 7 days</option>
<option value="30d">Past 30 days</option>
<option value="cm">Current month</option>
<option value="lm">Last month</option>
<option value="3m">Last 3 months</option>
</select>
<label>Start <input type="date" name="start" value="{start}"></label>
<label>End <input type="date" name="end" value="{end}"></label>
<button type="submit">Apply</button>
</form>"#,
        action = html_escape(action),
        start = html_escape(start),
        end = html_escape(end),
    )
}

const COLLAPSE_THRESHOLD: usize = 200;

pub fn collapsible_block(content: &str, css_class: &str) -> String {
    let escaped = html_escape(content);
    if content.len() <= COLLAPSE_THRESHOLD {
        if content.contains('\n') {
            return format!(r#"<pre class="{}">{}</pre>"#, css_class, escaped);
        } else {
            return format!(r#"<div class="{}">{}</div>"#, css_class, escaped);
        }
    }
    let preview: String = content.chars().take(COLLAPSE_THRESHOLD).collect();
    let preview_escaped = html_escape(&preview);
    format!(
        r#"<details class="collapsible"><summary><span class="preview-text {cls}">{preview}...</span> <span class="show-more">show more</span><span class="show-less">show less</span></summary><div class="collapsible-full {cls}">{full}</div></details>"#,
        cls = css_class,
        preview = preview_escaped,
        full = escaped
    )
}

pub fn page_layout(title: &str, body_html: String) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
body {{ font-family: monospace; padding: 16px; }}
table {{ width: 100%; border-collapse: collapse; }}
th {{ text-align: left; padding: 6px 8px; border-bottom: 1px solid #ccc; cursor: pointer; user-select: none; }}
th:after {{ content: ' \2195 '; color: #ccc; }}
th.sort-asc:after {{ content: ' \25B2 '; color: #333; }}
th.sort-desc:after {{ content: ' \25BC '; color: #333; }}
td {{ padding: 6px 8px; border-bottom: 1px solid #eee; vertical-align: top; }}
tr:last-child td {{ border-bottom: none; }}
pre {{ white-space: pre-wrap; }}
form {{ display: inline; }}
details.collapsible {{ display: flex; flex-direction: column; }}
details.collapsible > summary {{ cursor: pointer; list-style: none; order: 1; }}
details.collapsible > summary::-webkit-details-marker {{ display: none; }}
details.collapsible > summary .show-less {{ display: none; }}
details.collapsible > .collapsible-full {{ white-space: pre-wrap; word-break: break-word; order: 0; }}
details.collapsible[open] > summary .preview-text {{ display: none; }}
details.collapsible[open] > summary .show-more {{ display: none; }}
details.collapsible[open] > summary .show-less {{ display: inline; }}
.hidden {{ display: none; }}
.filtered-row {{ opacity: 0.45; }}
.filtered-badge {{ color: #888; font-weight: bold; font-size: 0.85em; }}
.export-csv-btn {{ margin-bottom: 8px; cursor: pointer; font-family: monospace; padding: 4px 12px; }}
</style>
</head>
<body>
{body_html}
<script>
(function(){{
  document.querySelectorAll('th').forEach(function(th){{
    th.addEventListener('click',function(){{
      var table=th.closest('table'),idx=Array.prototype.indexOf.call(th.parentNode.children,th);
      var rows=Array.from(table.querySelectorAll('tr')).slice(1);
      if(!rows.length)return;
      var asc=!th.classList.contains('sort-asc');
      table.querySelectorAll('th').forEach(function(h){{h.classList.remove('sort-asc','sort-desc');}});
      th.classList.add(asc?'sort-asc':'sort-desc');
      rows.sort(function(a,b){{
        var at=(a.children[idx]||{{}}).textContent||'';
        var bt=(b.children[idx]||{{}}).textContent||'';
        var an=parseFloat(at),bn=parseFloat(bt);
        var r=(!isNaN(an)&&!isNaN(bn))?an-bn:at.localeCompare(bt);
        return asc?r:-r;
      }});
      rows.forEach(function(r){{table.appendChild(r);}});
    }});
  }});
}})();
(function(){{
  function exportCsv(table){{
    var name=table.getAttribute('data-export-name')||'cost_export';
    var rows=Array.from(table.querySelectorAll('tr'));
    var csv=rows.map(function(row){{
      return Array.from(row.querySelectorAll('th,td')).map(function(cell){{
        var text=(cell.textContent||'').replace(/"/g,'""');
        return '"'+text+'"';
      }}).join(',');
    }}).join('\n');
    var blob=new Blob([csv],{{type:'text/csv;charset=utf-8;'}});
    var url=URL.createObjectURL(blob);
    var a=document.createElement('a');
    var s=document.querySelector('input[name="start"]');
    var e=document.querySelector('input[name="end"]');
    var fname=name+(s&&s.value?'_'+s.value:'')+(e&&e.value?'_'+e.value:'')+'.csv';
    a.href=url;a.download=fname;a.style.display='none';
    document.body.appendChild(a);a.click();
    document.body.removeChild(a);URL.revokeObjectURL(url);
  }}
  document.querySelectorAll('table.data-table').forEach(function(table){{
    var btn=document.createElement('button');
    btn.textContent='Export CSV';btn.className='export-csv-btn';
    btn.addEventListener('click',function(){{exportCsv(table);}});
    table.parentNode.insertBefore(btn,table);
  }});
}})();
</script>
</body>
</html>"#,
        title = html_escape(title),
        body_html = body_html
    )
}

pub struct Breadcrumb {
    pub label: String,
    pub href: Option<String>,
}

impl Breadcrumb {
    pub fn link(label: impl ToString, href: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            href: Some(href.to_string()),
        }
    }

    pub fn current(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            href: None,
        }
    }
}

pub struct NavLink {
    pub label: String,
    pub href: String,
}

impl NavLink {
    pub fn new(label: impl ToString, href: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            href: href.to_string(),
        }
    }

    pub fn back() -> Self {
        Self {
            label: "Back".to_string(),
            href: "javascript:history.back()".to_string(),
        }
    }
}

pub struct InfoRow {
    pub label: String,
    pub value: String,
}

impl InfoRow {
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: html_escape(value),
        }
    }

    pub fn raw(label: &str, value: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
        }
    }
}

pub struct Subpage {
    pub label: String,
    pub href: String,
    pub count: String,
}

impl Subpage {
    pub fn new(label: impl ToString, href: impl ToString, count: impl std::fmt::Display) -> Self {
        Self {
            label: label.to_string(),
            href: href.to_string(),
            count: count.to_string(),
        }
    }
}

pub struct Page<C: IntoView = ()> {
    pub title: String,
    pub breadcrumbs: Vec<Breadcrumb>,
    pub nav_links: Vec<NavLink>,
    pub info_rows: Vec<InfoRow>,
    pub content: C,
    pub subpages: Vec<Subpage>,
}

impl Default for Page {
    fn default() -> Self {
        Page {
            title: String::new(),
            breadcrumbs: Vec::new(),
            nav_links: Vec::new(),
            info_rows: Vec::new(),
            content: (),
            subpages: Vec::new(),
        }
    }
}

impl<C: IntoView> Page<C> {
    pub fn render(self) -> String {
        let Page {
            title,
            breadcrumbs,
            nav_links,
            info_rows,
            content,
            subpages,
        } = self;

        let body = view! {
            {if !breadcrumbs.is_empty() {
                Either::Left(view! {
                    <h1>
                        {breadcrumbs.into_iter().enumerate().map(|(i, crumb)| {
                            let sep = if i > 0 { " / " } else { "" };
                            match crumb.href {
                                Some(href) => Either::Left(view! {
                                    {sep}<a href={href}>{crumb.label}</a>
                                }),
                                None => Either::Right(view! {
                                    {sep}{crumb.label}
                                }),
                            }
                        }).collect::<Vec<_>>()}
                    </h1>
                })
            } else {
                Either::Right(())
            }}

            {if !nav_links.is_empty() {
                Either::Left(view! {
                    <h2>"Navigation"</h2>
                    <table>
                        {nav_links.into_iter().map(|link| {
                            view! { <tr><td><a href={link.href}>{link.label}</a></td></tr> }
                        }).collect::<Vec<_>>()}
                    </table>
                })
            } else {
                Either::Right(())
            }}

            {if !info_rows.is_empty() {
                Either::Left(view! {
                    <h2>"Info"</h2>
                    <table>
                        {info_rows.into_iter().map(|row| {
                            view! { <tr><td>{row.label}</td><td inner_html={row.value}></td></tr> }
                        }).collect::<Vec<_>>()}
                    </table>
                })
            } else {
                Either::Right(())
            }}

            {content}

            {if !subpages.is_empty() {
                Either::Left(view! {
                    <h2>"Subpages"</h2>
                    <table>
                        <tr><th>"Page"</th><th>"Count"</th></tr>
                        {subpages.into_iter().map(|sp| {
                            view! { <tr><td><a href={sp.href}>{sp.label}</a></td><td>{sp.count}</td></tr> }
                        }).collect::<Vec<_>>()}
                    </table>
                })
            } else {
                Either::Right(())
            }}
        };

        page_layout(&title, body.to_html())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_escape_special_chars() {
        assert_eq!(
            html_escape("<b>\"a&b\"</b>"),
            "&lt;b&gt;&quot;a&amp;b&quot;&lt;/b&gt;"
        );
    }

    #[test]
    fn html_escape_no_special_chars() {
        assert_eq!(html_escape("hello world"), "hello world");
    }

    #[test]
    fn collapsible_block_short_single_line() {
        let result = collapsible_block("short text", "cls");
        assert_eq!(result, r#"<div class="cls">short text</div>"#);
    }

    #[test]
    fn collapsible_block_short_multiline() {
        let result = collapsible_block("line1\nline2", "cls");
        assert_eq!(
            result,
            r#"<pre class="cls">line1
line2</pre>"#
        );
    }

    #[test]
    fn collapsible_block_long_content() {
        let long = "a".repeat(300);
        let result = collapsible_block(&long, "cls");
        assert!(result.contains("show more"));
        assert!(result.contains("show less"));
        assert!(result.contains("collapsible"));
    }

    #[test]
    fn page_layout_wraps_body() {
        let result = page_layout("Test Title", "<p>body</p>".to_string());
        assert!(result.contains("<title>Test Title</title>"));
        assert!(result.contains("<p>body</p>"));
        assert!(result.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn page_layout_escapes_title() {
        let result = page_layout("<script>", "".to_string());
        assert!(result.contains("<title>&lt;script&gt;</title>"));
    }

    #[test]
    fn page_render_breadcrumbs_only() {
        let html = Page {
            title: "Test".to_string(),
            breadcrumbs: vec![
                Breadcrumb::link("Home", "/"),
                Breadcrumb::current("Current"),
            ],
            nav_links: vec![],
            info_rows: vec![],
            content: (),
            subpages: vec![],
        }
        .render();
        assert!(html.contains("<h1>"));
        assert!(html.contains(r#"<a href="/">"#));
        assert!(html.contains("Home"));
        assert!(html.contains(" / "));
        assert!(html.contains("Current"));
        assert!(html.contains("</h1>"));
    }

    #[test]
    fn page_render_nav_links() {
        let html = Page {
            title: "Test".to_string(),
            breadcrumbs: vec![],
            nav_links: vec![NavLink::new("Edit", "/edit"), NavLink::back()],
            info_rows: vec![],
            content: (),
            subpages: vec![],
        }
        .render();
        assert!(html.contains("<h2>Navigation</h2>"));
        assert!(html.contains(r#"<a href="/edit">"#));
        assert!(html.contains("Edit"));
        assert!(html.contains(r#"<a href="javascript:history.back()">"#));
        assert!(html.contains("Back"));
    }

    #[test]
    fn page_render_info_rows_escaped() {
        let html = Page {
            title: "Test".to_string(),
            breadcrumbs: vec![],
            nav_links: vec![],
            info_rows: vec![InfoRow::new("Key", "<value>")],
            content: (),
            subpages: vec![],
        }
        .render();
        assert!(html.contains("<h2>Info</h2>"));
        assert!(html.contains("Key"));
        assert!(html.contains("&lt;value&gt;"));
        assert!(!html.contains("<value>"));
    }

    #[test]
    fn page_render_info_rows_raw() {
        let html = Page {
            title: "Test".to_string(),
            breadcrumbs: vec![],
            nav_links: vec![],
            info_rows: vec![InfoRow::raw("Key", "<b>bold</b>")],
            content: (),
            subpages: vec![],
        }
        .render();
        assert!(html.contains("<b>bold</b>"));
    }

    #[test]
    fn page_render_content_view() {
        let html = Page {
            title: "Test".to_string(),
            breadcrumbs: vec![],
            nav_links: vec![],
            info_rows: vec![],
            content: view! { <form><input type="text" name="x"/></form> },
            subpages: vec![],
        }
        .render();
        assert!(html.contains("<form>"));
        assert!(html.contains(r#"name="x""#));
    }

    #[test]
    fn page_render_subpages() {
        let html = Page {
            title: "Test".to_string(),
            breadcrumbs: vec![],
            nav_links: vec![],
            info_rows: vec![],
            content: (),
            subpages: vec![Subpage::new("Requests", "/requests", 42)],
        }
        .render();
        assert!(html.contains("<h2>Subpages</h2>"));
        assert!(html.contains("Page"));
        assert!(html.contains("Count"));
        assert!(html.contains(r#"<a href="/requests">"#));
        assert!(html.contains("Requests"));
        assert!(html.contains("42"));
    }

    #[test]
    fn page_render_empty_sections_omitted() {
        let html = Page {
            title: "Test".to_string(),
            breadcrumbs: vec![],
            nav_links: vec![],
            info_rows: vec![],
            content: (),
            subpages: vec![],
        }
        .render();
        assert!(!html.contains("<h1>"));
        assert!(!html.contains("Navigation"));
        assert!(!html.contains("Info"));
        assert!(!html.contains("Subpages"));
    }

    #[test]
    fn page_render_full() {
        let html = Page {
            title: "Full Page".to_string(),
            breadcrumbs: vec![Breadcrumb::link("Home", "/"), Breadcrumb::current("Detail")],
            nav_links: vec![NavLink::back()],
            info_rows: vec![InfoRow::new("Name", "test")],
            content: view! { <p>"content"</p> },
            subpages: vec![Subpage::new("Sub", "/sub", 5)],
        }
        .render();
        assert!(html.contains("<title>Full Page</title>"));
        assert!(html.contains("<h1>"));
        assert!(html.contains("Navigation"));
        assert!(html.contains("Info"));
        assert!(html.contains("<p>"));
        assert!(html.contains("content"));
        assert!(html.contains("Subpages"));
    }
}
