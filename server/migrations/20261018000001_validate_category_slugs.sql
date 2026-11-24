ALTER TABLE categories
    ADD CONSTRAINT categories_slug_format_check
    CHECK (slug ~ '^[a-z0-9-]+$');

CREATE UNIQUE INDEX categories_slug_lower_unique
    ON categories (LOWER(slug));