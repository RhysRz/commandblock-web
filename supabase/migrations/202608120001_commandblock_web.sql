create table public.profiles (
  id uuid primary key references auth.users(id) on delete cascade,
  display_name text not null default 'ผู้ใช้ Commandblock',
  avatar_url text,
  created_at timestamptz not null default now()
);

create table public.conversations (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade,
  title text not null default 'แชทใหม่',
  model_id text not null default 'deepseek-v4-flash',
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table public.messages (
  id uuid primary key default gen_random_uuid(),
  conversation_id uuid not null references public.conversations(id) on delete cascade,
  user_id uuid not null references auth.users(id) on delete cascade,
  role text not null check (role in ('user', 'assistant')),
  content text not null,
  created_at timestamptz not null default now()
);

alter table public.profiles enable row level security;
alter table public.conversations enable row level security;
alter table public.messages enable row level security;

create policy users_manage_own_profiles on public.profiles for all
  using ((select auth.uid()) = id) with check ((select auth.uid()) = id);
create policy users_manage_own_conversations on public.conversations for all
  using ((select auth.uid()) = user_id) with check ((select auth.uid()) = user_id);
create policy users_manage_own_messages on public.messages for all
  using ((select auth.uid()) = user_id) with check ((select auth.uid()) = user_id);

create index conversations_user_updated_idx on public.conversations (user_id, updated_at desc);
create index messages_conversation_created_idx on public.messages (conversation_id, created_at);

create function public.handle_new_user() returns trigger
language plpgsql security definer set search_path = public
as $$
begin
  insert into public.profiles (id, display_name)
  values (new.id, coalesce(new.raw_user_meta_data ->> 'full_name', split_part(new.email, '@', 1), 'ผู้ใช้ Commandblock'));
  return new;
end;
$$;

create trigger on_auth_user_created
  after insert on auth.users for each row execute procedure public.handle_new_user();
