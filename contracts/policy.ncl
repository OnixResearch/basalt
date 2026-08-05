let NonEmptyString = std.contract.from_validator (fun value =>
  if std.is_string value && std.string.length value > 0 then
    'Ok
  else
    'Error { message = "expected non-empty string" }
) in
let ContractPolicy = {
  id | NonEmptyString,
  description | String | default = "",
  resource_prefixes | Array NonEmptyString,
  abilities | Array NonEmptyString,
} in
let BackendPolicy = {
  id | NonEmptyString,
  description | String | default = "",
  supports_higher_order | Bool | default = false,
} in
let Policy = {
  schema_version | NonEmptyString,
  backends | { _ : BackendPolicy } | default = {},
  contracts | { _ : ContractPolicy },
} in
{
  schema_version = "ucan-nickel-contracts.policy.v1",
  backends = {
    nickel = {
      id = "nickel",
      description = "Nickel-authored static policy contracts.",
      supports_higher_order = false,
    },
    steel = {
      id = "steel",
      description = "Steel Scheme dynamic predicate contracts.",
      supports_higher_order = true,
    },
  },
  contracts = {
    document-read = {
      id = "document-read",
      description = "Read bounded document resources.",
      resource_prefixes = ["did:key:z6Mkresource/docs/"],
      abilities = ["document/read"],
    },
    document-write = {
      id = "document-write",
      description = "Write bounded document resources.",
      resource_prefixes = ["did:key:z6Mkresource/docs/"],
      abilities = ["document/write"],
    },
    onixos-target-candidate-activation = {
      id = "onixos-target-candidate-activation",
      description = "Activate one OnixOS candidate target.",
      resource_prefixes = ["did:key:z6Mkonix/onixos/"],
      abilities = ["onix/target-candidate-activation"],
    },
    onixos-target-generation-persistence = {
      id = "onixos-target-generation-persistence",
      description = "Persist one OnixOS generation.",
      resource_prefixes = ["did:key:z6Mkonix/onixos/"],
      abilities = ["onix/target-generation-persistence"],
    },
    onixos-target-rollback = {
      id = "onixos-target-rollback",
      description = "Roll back one OnixOS target.",
      resource_prefixes = ["did:key:z6Mkonix/onixos/"],
      abilities = ["onix/target-rollback"],
    },
    onixos-state-sync-publish = {
      id = "onixos-state-sync-publish",
      description = "Publish one OnixOS state-sync update.",
      resource_prefixes = ["onix://state-sync/"],
      abilities = ["state_sync.publish"],
    },
    onixos-state-sync-delete = {
      id = "onixos-state-sync-delete",
      description = "Delete one OnixOS state-sync update.",
      resource_prefixes = ["onix://state-sync/"],
      abilities = ["state_sync.delete"],
    },
    onixos-state-sync-admin = {
      id = "onixos-state-sync-admin",
      description = "Administer one OnixOS state-sync update.",
      resource_prefixes = ["onix://state-sync/"],
      abilities = ["state_sync.admin"],
    },
  },
} | Policy
