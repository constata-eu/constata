use const_format::formatcp;

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
  adminAccessUrl
  isPublished
  publicCertificateUrl
  __typename";

pub const UNSIGNED_ENTRY_PAYLOAD: &'static str = formatcp!("\
  id
  entry {{
    {ENTRY}
  }}
  bytes
  __typename
");

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
  publicCertificateUrl
  createdAt
  documents {
    certificationDate
    parts {
        friendlyName
        hash
        isBase
        signatures {
            certificationDate
            publicKey
            signature
            signatureHash
            endorsementManifest {
                text
                websites
                kyc {
                    name
                    lastName
                    idNumber
                    idType
                    birthdate
                    nationality
                    country
                    jobTitle
                    legalEntityName
                    legalEntityCountry
                    legalEntityRegistration
                    legalEntityTaxId
                    updatedAt
                }
            }
        }
    }
  }
  __typename";

pub const ACCOUNT_STATE: &'static str = "\
  id
  missing
  tokenBalance
  pricePerToken
  maxMonthlyGift
  monthlyGiftRemainder
  parkedCount
  invoices {
    amount
    tokens
    description
    url
    __typename
  }
  pendingTycUrl
  pendingInvoiceLinkUrl
  webCallbacksUrl
  __typename";

pub const WEB_CALLBACK: &'static str = "\
  id
  kind
  resourceId
  state
  lastAttemptId
  createdAt
  nextAttemptOn
  requestBody
  __typename";
