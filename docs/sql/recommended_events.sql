-- ============================================================
-- Agora: Recommended Events Query
-- Strategy: Content-based filtering via category_id overlap
--           from the user's 3 most recent ticket purchases.
-- ============================================================

-- Step 1: Derive the categories from the user's last 3 purchases
WITH recent_purchases AS (
  SELECT DISTINCT e.category_id
  FROM   tickets      t
  JOIN   events       e ON e.id = t.event_id
  WHERE  t.user_id   = $1                    -- :user_id
    AND  t.status    = 'confirmed'
  ORDER  BY t.created_at DESC
  LIMIT  3
),

-- Step 2: Score candidate events by how many purchased categories they match
scored_events AS (
  SELECT
    e.id,
    e.title,
    e.slug,
    e.description,
    e.start_time,
    e.end_time,
    e.location,
    e.banner_url,
    e.category_id,
    c.name          AS category_name,
    e.organizer_id,
    u.display_name  AS organizer_name,
    u.avatar_url    AS organizer_avatar,

    -- Minimum ticket price for this event (NULL → free)
    (
      SELECT MIN(tp.price)
      FROM   ticket_types tp
      WHERE  tp.event_id = e.id
        AND  tp.is_active = TRUE
    ) AS min_price,

    -- Tickets remaining across all active ticket types
    (
      SELECT COALESCE(SUM(tp.quantity - tp.sold), 0)
      FROM   ticket_types tp
      WHERE  tp.event_id = e.id
        AND  tp.is_active = TRUE
    ) AS tickets_remaining,

    -- Relevance: count of matching purchased categories (≥1 guaranteed by JOIN)
    COUNT(rp.category_id) AS relevance_score

  FROM   events        e
  JOIN   categories    c  ON c.id = e.category_id
  JOIN   users         u  ON u.id = e.organizer_id
  JOIN   recent_purchases rp ON rp.category_id = e.category_id

  WHERE  e.status      = 'published'
    AND  e.start_time  > NOW()               -- future events only

    -- Exclude events the user already has a ticket for
    AND  e.id NOT IN (
           SELECT t2.event_id
           FROM   tickets t2
           WHERE  t2.user_id = $1
             AND  t2.status  = 'confirmed'
         )

  GROUP  BY
    e.id, e.title, e.slug, e.description,
    e.start_time, e.end_time, e.location,
    e.banner_url, e.category_id, c.name,
    e.organizer_id, u.display_name, u.avatar_url
)

SELECT *
FROM   scored_events
ORDER  BY
  relevance_score DESC,   -- most category overlap first
  start_time      ASC     -- then soonest upcoming
LIMIT  12;