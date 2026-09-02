//! Brain tools: create, read, update, delete, search, link, tag, and manage brain pages
//! via the AI function registry.

use crate::brain::BrainManager;
use crate::engine::tool_registry::{ToolDef, ToolRegistry};
use std::sync::Arc;

/// Register brain-related tools with the registry.
pub fn register(registry: &ToolRegistry, brain: Arc<BrainManager>) {
    // brain_create_page
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_create_page".into(),
        description: "Create a new brain page for an entity (person, company, project, concept, meeting, idea, daily, inbox)".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "page_type": {"type": "string", "description": "Entity type: person, company, project, concept, meeting, idea, daily, inbox"},
                "title": {"type": "string", "description": "Page title (e.g., 'Sarah Chen', 'Anthropic')"},
                "content": {"type": "string", "description": "Initial compiled truth content (markdown)"}
            },
            "required": ["page_type", "title", "content"]
        }),
        handler: Box::new(move |params| {
            let page_type = params["page_type"].as_str().ok_or("page_type is required")?;
            let title = params["title"].as_str().ok_or("title is required")?;
            let content = params["content"].as_str().unwrap_or("");
            let page = bm.create_page(page_type, title, content)?;
            Ok(format!("Created brain page '{}' at {}", page.title, page.slug))
        }),
    });

    // brain_read_page
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_read_page".into(),
        description: "Read a brain page by its slug (e.g., 'people/sarah-chen')".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {"type": "string", "description": "Page slug (e.g., 'people/sarah-chen')"}
            },
            "required": ["slug"]
        }),
        handler: Box::new(move |params| {
            let slug = params["slug"].as_str().ok_or("slug is required")?;
            match bm.read_page(slug)? {
                Some(page) => serde_json::to_string_pretty(&page)
                    .map_err(|e| format!("Serialization error: {e}")),
                None => Ok(format!("Page not found: {slug}")),
            }
        }),
    });

    // brain_update_page
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_update_page".into(),
        description: "Update a brain page's compiled truth content. Creates a version snapshot first.".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {"type": "string", "description": "Page slug"},
                "content": {"type": "string", "description": "New compiled truth content (markdown)"}
            },
            "required": ["slug", "content"]
        }),
        handler: Box::new(move |params| {
            let slug = params["slug"].as_str().ok_or("slug is required")?;
            let content = params["content"].as_str().ok_or("content is required")?;
            let page = bm.update_page(slug, content)?;
            Ok(format!("Updated brain page '{}'", page.title))
        }),
    });

    // brain_delete_page
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_delete_page".into(),
        description: "Soft-delete a brain page (moves to archive)".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {"type": "string", "description": "Page slug to delete"}
            },
            "required": ["slug"]
        }),
        handler: Box::new(move |params| {
            let slug = params["slug"].as_str().ok_or("slug is required")?;
            bm.delete_page(slug)?;
            Ok(format!("Deleted brain page '{slug}' (moved to archive)"))
        }),
    });

    // brain_list_pages
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_list_pages".into(),
        description: "List brain pages, optionally filtered by entity type".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "page_type": {"type": "string", "description": "Filter by type (person, company, project, etc.)"},
                "limit": {"type": "number", "description": "Max results (default 50)"}
            }
        }),
        handler: Box::new(move |params| {
            let page_type = params["page_type"].as_str();
            let limit = params["limit"].as_u64().map(|n| n as usize);
            let pages = bm.list_pages(page_type, limit)?;
            if pages.is_empty() {
                return Ok("No brain pages found.".to_string());
            }
            let lines: Vec<String> = pages
                .iter()
                .map(|p| format!("[{}] {} — {}", p.page_type, p.title, p.slug))
                .collect();
            Ok(lines.join("\n"))
        }),
    });

    // brain_search
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_search".into(),
        description: "Search brain pages using full-text search (FTS5). Returns ranked results with snippets.".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "limit": {"type": "number", "description": "Max results (default 10)"},
                "page_type": {"type": "string", "description": "Filter by entity type"}
            },
            "required": ["query"]
        }),
        handler: Box::new(move |params| {
            let query = params["query"].as_str().ok_or("query is required")?;
            let limit = params["limit"].as_u64().map(|n| n as usize);
            let page_type = params["page_type"].as_str();
            let results = bm.search(query, limit, page_type)?;
            if results.is_empty() {
                return Ok(format!("No brain pages found matching '{query}'."));
            }
            let lines: Vec<String> = results
                .iter()
                .map(|r| format!("[{}] {} ({}) — {}", r.page_type, r.title, r.slug, r.snippet))
                .collect();
            Ok(lines.join("\n"))
        }),
    });

    // brain_add_timeline
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_add_timeline".into(),
        description: "Add a timeline entry to a brain page (append-only event log)".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {"type": "string", "description": "Page slug"},
                "date": {"type": "string", "description": "ISO date (YYYY-MM-DD)"},
                "source": {"type": "string", "description": "Event source (conversation, meeting, manual, email)"},
                "summary": {"type": "string", "description": "Short summary of the event"},
                "detail": {"type": "string", "description": "Detailed content (optional)"}
            },
            "required": ["slug", "date", "source", "summary"]
        }),
        handler: Box::new(move |params| {
            let slug = params["slug"].as_str().ok_or("slug is required")?;
            let date = params["date"].as_str().ok_or("date is required")?;
            let source = params["source"].as_str().ok_or("source is required")?;
            let summary = params["summary"].as_str().ok_or("summary is required")?;
            let detail = params["detail"].as_str();
            let id = bm.add_timeline(slug, date, source, summary, detail)?;
            Ok(format!("Added timeline entry #{id} to {slug}"))
        }),
    });

    // brain_get_timeline
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_get_timeline".into(),
        description: "Get timeline entries for a brain page, ordered by date".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {"type": "string", "description": "Page slug"},
                "limit": {"type": "number", "description": "Max entries (default 50)"}
            },
            "required": ["slug"]
        }),
        handler: Box::new(move |params| {
            let slug = params["slug"].as_str().ok_or("slug is required")?;
            let limit = params["limit"].as_u64().map(|n| n as usize);
            let entries = bm.get_timeline(slug, limit)?;
            if entries.is_empty() {
                return Ok(format!("No timeline entries for {slug}."));
            }
            let lines: Vec<String> = entries
                .iter()
                .map(|e| format!("### {} — {}\n{}", e.date, e.source, e.summary))
                .collect();
            Ok(lines.join("\n\n"))
        }),
    });

    // brain_add_link
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_add_link".into(),
        description: "Create a typed link between two brain pages".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "from_slug": {"type": "string", "description": "Source page slug"},
                "to_slug": {"type": "string", "description": "Target page slug"},
                "link_type": {"type": "string", "description": "Link type (reference, works_at, invested_in, related_to)"},
                "context": {"type": "string", "description": "Context for the link (optional)"}
            },
            "required": ["from_slug", "to_slug"]
        }),
        handler: Box::new(move |params| {
            let from = params["from_slug"].as_str().ok_or("from_slug is required")?;
            let to = params["to_slug"].as_str().ok_or("to_slug is required")?;
            let link_type = params["link_type"].as_str();
            let context = params["context"].as_str();
            bm.add_link(from, to, link_type, context)?;
            Ok(format!("Linked {from} → {to}"))
        }),
    });

    // brain_remove_link
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_remove_link".into(),
        description: "Remove a link between two brain pages".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "from_slug": {"type": "string", "description": "Source page slug"},
                "to_slug": {"type": "string", "description": "Target page slug"}
            },
            "required": ["from_slug", "to_slug"]
        }),
        handler: Box::new(move |params| {
            let from = params["from_slug"]
                .as_str()
                .ok_or("from_slug is required")?;
            let to = params["to_slug"].as_str().ok_or("to_slug is required")?;
            bm.remove_link(from, to)?;
            Ok(format!("Removed link {from} → {to}"))
        }),
    });

    // brain_get_links
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_get_links".into(),
        description: "Get all outbound links and backlinks for a brain page".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {"type": "string", "description": "Page slug"}
            },
            "required": ["slug"]
        }),
        handler: Box::new(move |params| {
            let slug = params["slug"].as_str().ok_or("slug is required")?;
            let links = bm.get_links(slug)?;
            let mut lines = Vec::new();
            if !links.outbound.is_empty() {
                lines.push("## Outbound Links".to_string());
                for l in &links.outbound {
                    lines.push(format!("→ {} [{}] {}", l.to_slug, l.link_type, l.context));
                }
            }
            if !links.backlinks.is_empty() {
                lines.push("## Backlinks".to_string());
                for l in &links.backlinks {
                    lines.push(format!("← {} [{}] {}", l.from_slug, l.link_type, l.context));
                }
            }
            if lines.is_empty() {
                return Ok(format!("No links for {slug}."));
            }
            Ok(lines.join("\n"))
        }),
    });

    // brain_add_tag
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_add_tag".into(),
        description: "Add a tag to a brain page".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {"type": "string", "description": "Page slug"},
                "tag": {"type": "string", "description": "Tag to add"}
            },
            "required": ["slug", "tag"]
        }),
        handler: Box::new(move |params| {
            let slug = params["slug"].as_str().ok_or("slug is required")?;
            let tag = params["tag"].as_str().ok_or("tag is required")?;
            bm.add_tag(slug, tag)?;
            Ok(format!("Added tag '{tag}' to {slug}"))
        }),
    });

    // brain_remove_tag
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_remove_tag".into(),
        description: "Remove a tag from a brain page".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {"type": "string", "description": "Page slug"},
                "tag": {"type": "string", "description": "Tag to remove"}
            },
            "required": ["slug", "tag"]
        }),
        handler: Box::new(move |params| {
            let slug = params["slug"].as_str().ok_or("slug is required")?;
            let tag = params["tag"].as_str().ok_or("tag is required")?;
            bm.remove_tag(slug, tag)?;
            Ok(format!("Removed tag '{tag}' from {slug}"))
        }),
    });

    // brain_stats
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_stats".into(),
        description: "Get brain statistics: page count, links, tags, timeline entries".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        handler: Box::new(move |_| {
            let stats = bm.stats()?;
            let mut lines = vec![
                format!("Pages: {}", stats.page_count),
                format!("Links: {}", stats.link_count),
                format!("Tags: {}", stats.tag_count),
                format!("Timeline entries: {}", stats.timeline_count),
            ];
            if !stats.pages_by_type.is_empty() {
                lines.push("\nBy type:".to_string());
                let mut types: Vec<_> = stats.pages_by_type.iter().collect();
                types.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
                for (t, count) in types {
                    lines.push(format!("  {t}: {count}"));
                }
            }
            Ok(lines.join("\n"))
        }),
    });

    // brain_resolve
    let bm = Arc::clone(&brain);
    registry.register(ToolDef {
        name: "brain_resolve".into(),
        description: "Resolve a partial name to matching brain page slugs (searches aliases and titles)".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Partial name to resolve (e.g., 'Sarah', 'acme')"}
            },
            "required": ["query"]
        }),
        handler: Box::new(move |params| {
            let query = params["query"].as_str().ok_or("query is required")?;
            let slugs = bm.resolve_slug(query)?;
            if slugs.is_empty() {
                return Ok(format!("No brain pages matching '{query}'."));
            }
            Ok(slugs.join("\n"))
        }),
    });
}
