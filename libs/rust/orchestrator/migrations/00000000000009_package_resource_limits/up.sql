alter table packages
    add column cpu_limit_millicores bigint null,
    add column memory_limit_mb bigint null;
