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
  },
} | Policy
