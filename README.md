Constata is an open-source blockchain stamping engine.

- [ ] SIOPv2 validation for validated ID responses.
- [ ] Support for did:ethr address as verification method in EcdsaSecp256k1Signature2019.

## Public api
- Can we remove public_api/src/controllers/static_files.rs ?

## Server side verifier tools.
- Use the organization logo for the dashboard, instead of constata's.

## Other 
- Make sqlx-models-derive support better transactions (start the transaction easier).
- Send PR to pdf library author.

- Refactor 'mod.rs' style modules.

# Client side stuff:
  - Translations.
  - Custom logo in menu.
  - Companies logo in the footer.
  - Red check for failed verifications.
  - Lists should show "empty" versions.
  - New credential dialog should be bigger with full width inputs.
  - Task to clean up old pending vc_requests.
  - Do not show pending or cancelled vc_requests in the list.
  - Show a "last used" field for vc_prompts. In the UI make it "In use now" if the date is less than 2 minutes ago.

  - Fix token authentication. (use two clients?)

# Admin: 
  - VcRequirements admin.
  - Refactor redundant admin API endpoints and react resources.
