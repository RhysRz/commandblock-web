-- Durable, owner-scoped SESSION pins. The existing conversations RLS policy
-- already restricts all updates and reads to the authenticated row owner.
alter table public.conversations
  add column if not exists is_pinned boolean not null default false;

-- Matches the owner filter and pinned-first session list ordering.
create index if not exists conversations_user_pin_updated_idx
  on public.conversations (user_id, is_pinned desc, updated_at desc, id desc);
