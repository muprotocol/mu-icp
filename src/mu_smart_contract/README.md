# mu Smart Contract Canister: Source of Truth and Master Data

This canister serves as the central source of truth and stores master data
for entities like developers, applications, escrow accounts, and usage of
additional services such as KV Store, Blob Storage, and more.

## Gateway for Developer Control

The canister acts as a gateway between developers and their application code.
It empowers developers with control over their applications, including
managing usage and other aspects.

## Current Services

- **Register Developer**: This service registers new developers and creates
a dedicated ledger sub-account as their escrow account.
This account tracks usage charges associated with additional canister
services employed within their applications.
- **Get Developer**: This service retrieves information about a registered
developer.
- **Deploy App**: This service uploads and stores the application code
(serialized along with the manifest file, facilitated by the mu CLI or mu
Dashboard website). Note: Currently, deployment of the app as a canister
on the ICP network is not supported. This functionality will be available
upon completion of the second milestone ("mu manager canister").
- **Remove App**: This service allows removing an application; however,
similar to deployment, app undeployment from the ICP network is not supported yet.
This functionality awaits the completion of the "mu runtime canister" milestone.
- **Get App/Get Apps**: This service retrieves applications submitted by
a specific developer. Apps can be in either an Active or Deleted state.
- **Request Escrow Withdraw**: This service allows developers to
  withdraw ICP tokens they previously deposited into their escrow
  account. 
- **Request Cycles (Exclusive to Apps)**: This service allows apps to
  request more cycles be transferred for them. This functionality allows
  a developer to have one escrow account filled with ICP tokens and a
  lot of apps that just request cycles as the go.


## Future Services

As the grant progresses and other components are developed, the following
services will be implemented:

- **Report Usage(Exclusive to mu Manager canister)**: This function, utilized by the "mu runtime canister",
  reports the usage of additional services and their canisters and top-up the
  cost of canisters using the estimated usage from their escrow account.
  This allows for automatic deductions from the escrow balance to cover
  associated costs. Additionally, it facilitates service termination in cases
  where the escrow account balance reaches zero.
- **Deploy App**: Same **Deploy App** service but with upgrading apps
  functionality, and actually deploying apps as canisters.
