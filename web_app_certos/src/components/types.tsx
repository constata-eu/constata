export interface Credentials {
    public_key?: string | null,
    encrypted_key?: string | null,
    environment?: string | null,
    address?: string | null,
}

export interface EmailAddressSection {
    address: string,
    keepPrivate: boolean,
    verifiedAt?: string,
}
  