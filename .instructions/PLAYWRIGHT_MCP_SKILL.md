# Playwright MCP Browser Automation

Use the Playwright MCP server for browser automation tasks including web scraping, form filling, testing, and interactive web workflows.

## When to Use

- **Web scraping**: Extract data from websites, capture screenshots, gather information
- **Form automation**: Fill out forms, submit data, handle file uploads
- **Testing workflows**: Verify web application behavior, check UI elements
- **Interactive tasks**: Login to websites, navigate multi-step processes
- **Documentation**: Capture screenshots of web pages or specific elements

## MCP Server Configuration

Add to your Claude Code MCP settings (`~/.claude.json` or project `.mcp.json`):

```json
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": ["@playwright/mcp@latest"]
    }
  }
}
```

For headed mode (visible browser):
```json
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": ["@playwright/mcp@latest", "--headed"]
    }
  }
}
```

## Available Tools

### Navigation

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| `browser_navigate` | Navigate to a URL | `url` (required) |
| `browser_navigate_back` | Go back to previous page | None |
| `browser_tabs` | List, create, close, or select tabs | `action`: "list", "new", "close", "select"; `index` for close/select |

### Page Analysis

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| `browser_snapshot` | Get accessibility tree of page (preferred for analysis) | `filename` (optional, save to file) |
| `browser_take_screenshot` | Capture visual screenshot | `fullPage`, `element`/`ref`, `filename`, `type` (png/jpeg) |
| `browser_console_messages` | Get browser console output | `level`: "error", "warning", "info", "debug" |
| `browser_network_requests` | Get all network requests | `includeStatic` (default false) |

### Interactions

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| `browser_click` | Click an element | `element` (description), `ref` (from snapshot), `button`, `doubleClick`, `modifiers` |
| `browser_type` | Type text into element | `element`, `ref`, `text`, `slowly`, `submit` |
| `browser_hover` | Hover over element | `element`, `ref` |
| `browser_press_key` | Press keyboard key | `key` (e.g., "Enter", "ArrowDown", "a") |
| `browser_select_option` | Select dropdown option | `element`, `ref`, `values` (array) |
| `browser_drag` | Drag and drop | `startElement`, `startRef`, `endElement`, `endRef` |

### Forms

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| `browser_fill_form` | Fill multiple form fields | `fields` array with `name`, `type`, `ref`, `value` |
| `browser_file_upload` | Upload files | `paths` (array of absolute file paths) |

### Advanced

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| `browser_evaluate` | Run JavaScript on page | `function`, optional `element`/`ref` |
| `browser_wait_for` | Wait for condition | `text`, `textGone`, or `time` (seconds) |
| `browser_handle_dialog` | Handle alert/confirm/prompt | `accept` (boolean), `promptText` |
| `browser_resize` | Resize browser window | `width`, `height` |
| `browser_run_code` | Execute Playwright code | `code` (async function with `page` argument) |

### Session Management

| Tool | Description |
|------|-------------|
| `browser_close` | Close the browser |
| `browser_install` | Install browser if missing |

## Common Workflows

### Login Flow

```
1. browser_navigate to login page
2. browser_snapshot to find form elements
3. browser_type username into username field (use ref from snapshot)
4. browser_type password into password field
5. browser_click the login/submit button
6. browser_wait_for success indicator or new page element
7. browser_snapshot to verify logged in state
```

### Form Submission

```
1. browser_navigate to form page
2. browser_snapshot to identify all form fields
3. browser_fill_form with all field values:
   - textbox: text inputs
   - checkbox: true/false
   - radio: selected value
   - combobox: dropdown selection text
   - slider: numeric value
4. browser_click submit button
5. browser_wait_for confirmation
```

### Data Extraction

```
1. browser_navigate to target page
2. browser_snapshot to get page structure
3. Use element refs to identify data locations
4. browser_evaluate to extract specific data with JavaScript
5. For multiple pages: browser_click pagination, repeat
```

### Screenshot Documentation

```
1. browser_navigate to page
2. browser_resize to desired dimensions (optional)
3. browser_take_screenshot for viewport or fullPage
4. For specific element: use element/ref parameters
```

## Tips

1. **Always use `browser_snapshot` first** - It returns element refs needed for interactions
2. **Use refs from snapshots** - The `ref` parameter must match exactly from the snapshot output
3. **Wait for page loads** - Use `browser_wait_for` after navigation or clicks that trigger loads
4. **Handle dialogs proactively** - Some sites show alerts; use `browser_handle_dialog`
5. **Check console for errors** - Use `browser_console_messages` if something isn't working
6. **Prefer snapshot over screenshot** - Snapshots are more useful for understanding page structure

## Error Handling

- **Browser not installed**: Call `browser_install` first
- **Element not found**: Re-run `browser_snapshot` to get current refs
- **Timeout errors**: Page may still be loading; use `browser_wait_for`
- **Click intercepted**: Element may be covered; try `browser_evaluate` with JavaScript click

## Example: Complete Login and Extract Data

```
1. browser_navigate({ url: "https://example.com/login" })
2. browser_snapshot({}) -> Find username field ref="input1", password ref="input2", button ref="button1"
3. browser_type({ element: "username field", ref: "input1", text: "myuser" })
4. browser_type({ element: "password field", ref: "input2", text: "mypass" })
5. browser_click({ element: "login button", ref: "button1" })
6. browser_wait_for({ text: "Welcome" })
7. browser_navigate({ url: "https://example.com/dashboard" })
8. browser_snapshot({}) -> Analyze dashboard structure
9. browser_evaluate({ function: "() => document.querySelector('.data-value').textContent" })
```
