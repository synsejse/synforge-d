create table users (
  id text primary key,
  handle text not null unique,
  display_name text not null,
  password_hash text not null,
  active integer not null default 1,
  created_at text not null,
  updated_at text not null
);

create table user_permissions (
  user_id text not null,
  permission text not null,
  primary key (user_id, permission)
);

create table user_repo_metrics (
  user_id text primary key,
  downloaded_bytes integer not null default 0,
  updated_at text not null
);

create index idx_user_permissions_user_id on user_permissions(user_id);
