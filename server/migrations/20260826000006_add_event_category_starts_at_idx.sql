-- Events are categorized through the event_categories join table.
CREATE INDEX IF NOT EXISTS idx_event_categories_category_event
    ON event_categories(category_id, event_id);
