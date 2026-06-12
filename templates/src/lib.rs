use leptos::either::Either;
use leptos::prelude::*;

/// Default cap on how many rows a CSV export gathers across pages.
pub const DEFAULT_EXPORT_ROW_CAP: usize = 1000;

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Renders the top navigation bar injected into every HTML page. It presents the view
/// modes, bolding the active one. `normal_label` names the default dashboard mode
/// ("Normal" in the user build, "Admin" in the admin build), `legacy_on` is the session's
/// legacy flag, `show_legacy` hides the Legacy mode when no map is configured (or in the
/// admin build), and `on_github` marks that the current page is the GitHub section.
pub fn top_bar(
    base_path: &str,
    normal_label: &str,
    legacy_on: bool,
    show_legacy: bool,
    on_github: bool,
) -> String {
    fn join(base: &str, suffix: &str) -> String {
        if suffix.is_empty() {
            return base.to_string();
        }
        format!("{}{}", base.trim_end_matches('/'), suffix)
    }
    fn item(active: bool, label: &str, href: String) -> String {
        if active {
            format!("<b>{}</b>", label)
        } else {
            format!(r#"<a href="{}">{}</a>"#, html_escape(&href), label)
        }
    }

    let normal_active = !on_github && !legacy_on;
    let legacy_active = !on_github && legacy_on;

    let mut items = vec![item(
        normal_active,
        normal_label,
        join(base_path, "/mode/normal"),
    )];
    if show_legacy {
        items.push(item(
            legacy_active,
            "Legacy",
            join(base_path, "/mode/legacy"),
        ));
    }
    items.push(item(on_github, "GitHub", join(base_path, "/costs/github")));

    format!(r#"<nav class="top-bar">{}</nav>"#, items.join(" "))
}

pub fn period_links(path: &str, active: &str) -> String {
    let periods = [
        ("7d", "Past 7 Days"),
        ("30d", "Past 30 Days"),
        ("month", "This Month"),
        ("last_month", "Last Month"),
        ("3m", "Last 3 Months"),
        ("6m", "Last 6 Months"),
        ("12m", "Last 12 Months"),
    ];
    let parts: Vec<String> = periods
        .iter()
        .map(|(key, label)| {
            if *key == active {
                format!("<b>{}</b>", html_escape(label))
            } else {
                let sep = if path.contains('?') { "&" } else { "?" };
                format!(
                    r#"<a href="{}{}period={}">{}</a>"#,
                    html_escape(path),
                    sep,
                    html_escape(key),
                    html_escape(label)
                )
            }
        })
        .collect();
    parts.join(" | ")
}

pub fn pagination_nav(path: &str, page: usize, total: usize, page_size: usize) -> String {
    if total <= page_size {
        return String::new();
    }
    let total_pages = total.div_ceil(page_size);
    let page = page.clamp(1, total_pages);
    let sep = if path.contains('?') { "&amp;" } else { "?" };
    let prev = if page > 1 {
        format!(
            r#"<a href="{}{}page={}">Prev</a>"#,
            html_escape(path),
            sep,
            page - 1
        )
    } else {
        "Prev".to_string()
    };
    let next = if page < total_pages {
        format!(
            r#"<a href="{}{}page={}">Next</a>"#,
            html_escape(path),
            sep,
            page + 1
        )
    } else {
        "Next".to_string()
    };
    format!(
        "{} | Page {} of {} ({} items) | {}",
        prev, page, total_pages, total, next
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

pub fn page_layout(title: &str, body_html: String, export_row_cap: usize) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
body {{ font-family: monospace; padding: 16px; }}
nav.top-bar {{ margin-bottom: 16px; text-align: center; }}
nav.top-bar a {{ margin: 0 8px; }}
table {{ width: 100%; border-collapse: collapse; }}
th {{ text-align: left; padding: 6px 8px; border-bottom: 1px solid #ccc; }}
table.data-table th {{ cursor: pointer; user-select: none; }}
table.data-table th:after {{ content: ' \2195 '; color: #ccc; }}
table.data-table th.sort-asc:after {{ content: ' \25B2 '; color: #333; }}
table.data-table th.sort-desc:after {{ content: ' \25BC '; color: #333; }}
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
  var params=new URLSearchParams(window.location.search);
  var curSort=params.get('sort');
  var curOrder=params.get('order')||'asc';
  // Mark sorted column header
  document.querySelectorAll('table.data-table').forEach(function(table){{
    var ths=table.querySelectorAll('tr:first-child th');
    if(curSort!==null){{
      var idx=parseInt(curSort,10);
      if(ths[idx])ths[idx].classList.add(curOrder==='desc'?'sort-desc':'sort-asc');
    }}
    // Click handler: navigate with sort params
    ths.forEach(function(th,i){{
      th.addEventListener('click',function(){{
        var p=new URLSearchParams(window.location.search);
        var newOrder=(p.get('sort')===String(i)&&p.get('order')!=='desc')?'desc':'asc';
        p.set('sort',i);p.set('order',newOrder);p.set('page','1');
        window.location.search=p.toString();
      }});
    }});
  }});
  // Append sort params to pagination and period links
  if(curSort!==null){{
    document.querySelectorAll('a[href]').forEach(function(a){{
      var h=a.getAttribute('href');
      if(h&&h.indexOf('sort=')===-1&&h.indexOf('?')!==-1)a.setAttribute('href',h+'&sort='+curSort+'&order='+curOrder);
    }});
  }}
}})();
(function(){{
  // Server paginates at 50 rows/page; export gathers across pages up to this cap.
  var PAGE_SIZE=50, MAX_ROWS={max_rows};
  function rowToCsv(row){{
    return Array.from(row.querySelectorAll('th,td')).map(function(cell){{
      var text=(cell.textContent||'').replace(/"/g,'""');
      return '"'+text+'"';
    }}).join(',');
  }}
  function download(table,lines){{
    var blob=new Blob([lines.join('\n')],{{type:'text/csv;charset=utf-8;'}});
    var url=URL.createObjectURL(blob);
    var a=document.createElement('a');
    var name=table.getAttribute('data-export-name')||'cost_export';
    var ds=table.getAttribute('data-start')||'';
    var de=table.getAttribute('data-end')||'';
    a.href=url;a.download=name+(ds?'_'+ds:'')+(de?'_'+de:'')+'.csv';
    a.style.display='none';
    document.body.appendChild(a);a.click();
    document.body.removeChild(a);URL.revokeObjectURL(url);
  }}
  function exportCurrentPage(table){{
    download(table,Array.from(table.querySelectorAll('tr')).map(rowToCsv));
  }}
  async function exportAll(table,btn){{
    var name=table.getAttribute('data-export-name');
    if(!name){{exportCurrentPage(table);return;}}
    var headerLine=null,dataLines=[],prevSig=null;
    var label=btn.textContent;btn.disabled=true;btn.textContent='Exporting...';
    try{{
      // Reuse current sort/order/period params; only vary the page.
      for(var p=1;p<=Math.ceil(MAX_ROWS/PAGE_SIZE)&&dataLines.length<MAX_ROWS;p++){{
        var params=new URLSearchParams(window.location.search);
        params.set('page',String(p));
        var resp=await fetch(window.location.pathname+'?'+params.toString(),{{credentials:'same-origin'}});
        if(!resp.ok)throw new Error('fetch failed: '+resp.status);
        var doc=new DOMParser().parseFromString(await resp.text(),'text/html');
        var t=Array.from(doc.querySelectorAll('table.data-table')).find(function(x){{
          return x.getAttribute('data-export-name')===name;
        }});
        if(!t)break;
        var trs=Array.from(t.querySelectorAll('tr'));
        var headers=trs.filter(function(r){{return r.querySelector('th');}});
        var body=trs.filter(function(r){{return !r.querySelector('th')&&r.querySelector('td');}});
        if(headerLine===null&&headers.length)headerLine=rowToCsv(headers[0]);
        // The server clamps out-of-range pages to the last page; identical
        // content means we've passed the end, so stop without duplicating.
        var sig=body.map(function(r){{return r.textContent;}}).join('|');
        if(sig===prevSig)break;
        prevSig=sig;
        body.forEach(function(r){{dataLines.push(rowToCsv(r));}});
        if(body.length<PAGE_SIZE)break;
      }}
      if(dataLines.length>MAX_ROWS)dataLines=dataLines.slice(0,MAX_ROWS);
      download(table,headerLine?[headerLine].concat(dataLines):dataLines);
    }}catch(e){{
      exportCurrentPage(table);
    }}finally{{
      btn.disabled=false;btn.textContent=label;
    }}
  }}
  document.querySelectorAll('table.data-table').forEach(function(table){{
    var btn=document.createElement('button');
    btn.textContent='Export CSV';btn.className='export-csv-btn';
    btn.addEventListener('click',function(){{exportAll(table,btn);}});
    table.parentNode.insertBefore(btn,table);
  }});
}})();
</script>
</body>
</html>"#,
        title = html_escape(title),
        body_html = body_html,
        max_rows = export_row_cap
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
    pub fn render(self, export_row_cap: usize) -> String {
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

        page_layout(&title, body.to_html(), export_row_cap)
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
        let result = page_layout(
            "Test Title",
            "<p>body</p>".to_string(),
            DEFAULT_EXPORT_ROW_CAP,
        );
        assert!(result.contains("<title>Test Title</title>"));
        assert!(result.contains("<p>body</p>"));
        assert!(result.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn page_layout_escapes_title() {
        let result = page_layout("<script>", "".to_string(), DEFAULT_EXPORT_ROW_CAP);
        assert!(result.contains("<title>&lt;script&gt;</title>"));
    }

    #[test]
    fn page_layout_embeds_export_row_cap() {
        // The CSV export JS must carry the configured cap.
        let result = page_layout("T", String::new(), 250);
        assert!(result.contains("MAX_ROWS=250"));
        assert!(result.contains("async function exportAll"));
        assert!(result.contains("getAttribute('data-export-name')"));
    }

    #[test]
    fn period_links_renders_active_bold() {
        let html = period_links("/users", "30d");
        assert!(html.contains("<b>Past 30 Days</b>"));
        assert!(!html.contains(r#"?period=30d"#));
    }

    #[test]
    fn period_links_renders_inactive_as_links() {
        let html = period_links("/users", "30d");
        assert!(html.contains(r#"<a href="/users?period=7d">Past 7 Days</a>"#));
        assert!(html.contains(r#"<a href="/users?period=month">This Month</a>"#));
        assert!(html.contains(r#"<a href="/users?period=last_month">Last Month</a>"#));
        assert!(html.contains(r#"<a href="/users?period=3m">Last 3 Months</a>"#));
    }

    #[test]
    fn period_links_separates_with_pipe() {
        let html = period_links("/", "7d");
        assert!(html.contains(" | "));
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
        .render(DEFAULT_EXPORT_ROW_CAP);
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
        .render(DEFAULT_EXPORT_ROW_CAP);
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
        .render(DEFAULT_EXPORT_ROW_CAP);
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
        .render(DEFAULT_EXPORT_ROW_CAP);
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
        .render(DEFAULT_EXPORT_ROW_CAP);
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
        .render(DEFAULT_EXPORT_ROW_CAP);
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
        .render(DEFAULT_EXPORT_ROW_CAP);
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
        .render(DEFAULT_EXPORT_ROW_CAP);
        assert!(html.contains("<title>Full Page</title>"));
        assert!(html.contains("<h1>"));
        assert!(html.contains("Navigation"));
        assert!(html.contains("Info"));
        assert!(html.contains("<p>"));
        assert!(html.contains("content"));
        assert!(html.contains("Subpages"));
    }
}
