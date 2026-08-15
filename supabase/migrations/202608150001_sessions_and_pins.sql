-- CommandBlock sessions use the existing conversations table.
-- The existing users_manage_own_messages policy enforces (select auth.uid()) = user_id.
alter table public.messages
  add column if not exists is_pinned boolean not null default false;

-- Supports owner-scoped session reads, pinned summary reads, and stable ordering.
create index if not exists messages_conversation_pin_created_idx
  on public.messages (user_id, conversation_id, is_pinned desc, created_at asc, id asc);
