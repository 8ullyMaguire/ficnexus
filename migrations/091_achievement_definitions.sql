-- Phase 9: Achievement definitions and unlock tracking
-- Defines achievements that unlock based on user actions

-- Ensure features table has achievement entries
INSERT INTO features (slug, name, description, feature_type, unlock_rank, unlock_trust, requires_feature)
VALUES
  ('first_post', 'First Post', 'Created your first forum post', 'achievement', 0, 0, NULL),
  ('popular_post', 'Popular Post', 'A post received 10+ reactions', 'achievement', 0, 0, NULL),
  ('forum_veteran', 'Forum Veteran', 'Created 50 forum posts', 'achievement', 0, 0, NULL),
  ('social_butterfly', 'Social Butterfly', 'Sent 25 direct messages', 'achievement', 0, 0, NULL),
  ('poll_master', 'Poll Master', 'Voted on 10 polls', 'achievement', 0, 0, NULL),
  ('reaction_giver', 'Helpful', 'Gave 25 reactions', 'achievement', 0, 0, NULL),
  ('level_5', 'Rising Star', 'Reached level 5', 'achievement', 0, 0, NULL),
  ('level_10', 'Established', 'Reached level 10', 'achievement', 0, 0, NULL),
  ('level_25', 'Veteran', 'Reached level 25', 'achievement', 0, 0, NULL),
  ('level_50', 'Legend', 'Reached level 50', 'achievement', 0, 0, NULL),
  ('level_100', 'Immortal', 'Reached level 100', 'achievement', 0, 0, NULL),
  ('mod_action', 'Moderator', 'Performed a moderation action', 'achievement', 0, 0, NULL),
  ('topic_creator', 'Conversation Starter', 'Created 10 topics', 'achievement', 0, 0, NULL),
  ('daily_visitor', 'Dedicated', 'Logged in 7 days in a row', 'achievement', 0, 0, NULL),
  ('night_owl', 'Night Owl', 'Posted at 3am local time', 'achievement', 0, 0, NULL),
  ('early_bird', 'Early Bird', 'Posted at 6am local time', 'achievement', 0, 0, NULL),
  ('weekend_warrior', 'Weekend Warrior', 'Posted on Saturday and Sunday', 'achievement', 0, 0, NULL),
  ('loremaster', 'Loremaster', 'Read 100 works', 'achievement', 0, 0, NULL),
  ('collector', 'Collector', 'Bookmarked 50 works', 'achievement', 0, 0, NULL),
  ('reviewer', 'Critic', 'Left 10 reviews', 'achievement', 0, 0, NULL)
ON CONFLICT (slug) DO UPDATE
  SET name = EXCLUDED.name,
      description = EXCLUDED.description;

-- Grant all level-based achievements to users who already qualify
INSERT INTO user_features (user_id, feature_id, unlocked_at)
SELECT u.id, f.id, NOW()
FROM users u
JOIN features f ON f.slug = CASE
  WHEN u.level >= 100 THEN 'level_100'
  WHEN u.level >= 50 THEN 'level_50'
  WHEN u.level >= 25 THEN 'level_25'
  WHEN u.level >= 10 THEN 'level_10'
  WHEN u.level >= 5 THEN 'level_5'
  ELSE NULL
END
WHERE u.level >= 5
ON CONFLICT DO NOTHING;

-- Grant first_post to users who already have posts
INSERT INTO user_features (user_id, feature_id, unlocked_at)
SELECT DISTINCT p.author_id, f.id, MIN(p.created_at)
FROM forum_posts p
JOIN features f ON f.slug = 'first_post'
GROUP BY p.author_id, f.id
ON CONFLICT DO NOTHING;

-- Grant topic_creator to users who already have 10+ topics
INSERT INTO user_features (user_id, feature_id, unlocked_at)
SELECT t.author_id, f.id, MIN(t.created_at)
FROM forum_topics t
JOIN features f ON f.slug = 'topic_creator'
GROUP BY t.author_id, f.id
HAVING COUNT(*) >= 10
ON CONFLICT DO NOTHING;

-- Grant popular_post to users whose posts have 10+ reactions
INSERT INTO user_features (user_id, feature_id, unlocked_at)
SELECT p.author_id, f.id, MIN(p.created_at)
FROM forum_posts p
JOIN features f ON f.slug = 'popular_post'
WHERE (
  SELECT COUNT(*) FROM forum_post_reactions r WHERE r.post_id = p.id
) >= 10
GROUP BY p.author_id, f.id
ON CONFLICT DO NOTHING;
