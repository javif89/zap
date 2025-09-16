# CSS Configuration

The WayCast UI is built on GTK, which means you can use GTK CSS to modify the styles how you want. Below
are the available options:

```css
/* Default CSS styles for Waycast launcher */
/* Not too much so that colors follow system theme */

/* Main window */
#launcher-window {}

/* Main container */
#main-container {
    padding: 16px;
}

/* Search input */
#search-input {
    font-size: 16px;
    padding: 12px;
    border-radius: 8px;
    margin-bottom: 12px;
}

#search-input:focus {
    box-shadow: 0 0 0 2px rgba(74, 144, 226, 0.3);
}

/* Results container */
#results-container {
    border-radius: 8px;
}

/* Results list */
#results-list {}

/* Individual list items */
#list-item {
    padding: 8px 12px;
    border-radius: 6px;
    margin: 2px;
}

#list-item:hover {}

#list-item:selected {}

/* Item icon */
#item-icon {
    margin-right: 12px;
}

/* Item text container */
#item-text {}

/* Item title */
#item-title {
    font-weight: 600;
    font-size: 14px;
}

/* Item description */
#item-description {
    font-size: 11px;
    margin-top: 2px;
}

/* CSS Classes (alternative to IDs) */

.launcher-window {
    /* Alternative window styling */
}

.launcher-search {
    /* Alternative search styling */
}

.launcher-item {
    /* Alternative item styling */
}

.launcher-item:hover {
    /* Alternative hover state */
}

.launcher-title {
    /* Alternative title styling */
}

.launcher-description {
    /* Alternative description styling */
}

.launcher-icon {
    /* Alternative icon styling */
}
```