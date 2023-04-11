pub const ISSUANCE: &'static str = "\
  id
  templateId
  templateName
  templateKind
  state
  name
  createdAt
  errors
  tokensNeeded
  entriesCount
  adminVisitedCount
  publicVisitCount
  __typename";

pub const ENTRY: &'static str = "\
  id
  issuanceId
  issuanceName
  rowNumber
  state
  receivedAt
  params
  errors
  documentId
  storyId
  adminVisited
  publicVisitCount
  hasEmailCallback
  emailCallbackSentAt
  payload
  adminAccessUrl
  __typename";

pub const TEMPLATE: &'static str = "\
  id
  name
  kind
  createdAt
  schema {
    name
    optional
    common
    label
    help
    sample
  }
  customMessage
  adminVisitedCount
  entriesCount
  publicVisitCount
  archived
  __typename";

pub const ATTESTATION: &'static str = "\
  id
  personId
  orgId
  markers
  openUntil
  state
  parkingReason
  doneDocuments
  parkedDocuments
  processingDocuments
  totalDocuments
  tokensCost
  tokensPaid
  tokensOwed
  buyTokensUrl
  acceptTycUrl
  lastDocDate
  emailAdminAccessUrlTo
  adminAccessUrl
  createdAt
  __typename";
