create table users (
  id varchar(36) primary key,
  handle varchar(255) not null unique,
  display_name varchar(255) not null,
  password_hash varchar(255) not null,
  active boolean not null default true,
  created_at varchar(64) not null,
  updated_at varchar(64) not null
);

create table user_permissions (
  user_id varchar(36) not null,
  permission varchar(32) not null,
  primary key (user_id, permission)
);

create table user_repo_metrics (
  user_id varchar(36) primary key,
  downloaded_bytes bigint not null default 0,
  updated_at varchar(64) not null
);

create index idx_user_permissions_user_id on user_permissions(user_id);
