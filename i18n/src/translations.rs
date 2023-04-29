macro_rules! make_translations {
  ( $( $name:ident: $en:literal $es:literal;)* ) => (
    pub struct TranslatedStrings {
      $(pub $name: &'static str,)*
    }

    pub const ENGLISH_STRINGS: TranslatedStrings = TranslatedStrings {
      $($name: $en,)*
    };
    pub const SPANISH_STRINGS: TranslatedStrings = TranslatedStrings {
      $($name: $es,)*
    };
  )
}

make_translations!{
  mailer_welcome_message_subject:
    "Hi welcome to Constata"
    "Hola, te doy la bienvenida a Constata";
  mailer_document_received_subject:
    "I got your document"
    "Recibí tu documento";
  template_message_for_diploma:
    "Hello {{ name }} this is your diploma for {{ motive }}."
    "Hola {{ name }}, este es tu diploma de {{ motive }}.";
  template_message_for_attendance:
    "Hello {{ name }} this is your certificate of attendance to {{ motive }}."
    "Hola {{ name }}, este es tu certificado de asistencia a {{ motive }}.";
  template_message_for_badge:
    "Hello {{ name }} this is a badge for {{ motive }}."
    "Hola {{ name }}, esta es una insignia por {{ motive }}.";
  mailer_email_callback_subject:
    "A certified document from {0}"
    "Un documento certificado de parte de {0}";
  mailer_parked_document_reminder_because_tyc_are_not_accepted_subject:
    "You have pending certifications you need to accept our Terms and Conditions."
    "Tienes certificaciones pendientes, necesitas aceptar nuestros términos y condiciones.";
  mailer_parked_document_reminder_because_payment_is_needed_subject:
    "You have pending certifications your payment is needed."
    "Tienes certificaciones pendientes, se necesita un pago.";
  mailer_welcome_after_website_signup_subject:
    "You have signed up to Constata.eu"
    "Te registraste a Constata.eu";
  mailer_kyc_request_acknowledge_subject:
    "We're working on verifying your identity."
    "Estamos trabajando en verificar tu identidad";
  mailer_email_address_verification_subject:
    "We need to verify this is your email"
    "Tenemos que verificar que este es tu email";
  public_certificate_share_text_diploma:
    "This diploma is certified by the Bitcoin blockchain!"
    "¡Este diploma está certificado en la blockchain de Bitcoin!";
  public_certificate_share_text_attendance:
    "This certificate of attendance is sealed by the Bitcoin blockchain!"
    "¡Este certificado de asistencia está sellado en la blockchain de Bitcoin!";
  public_certificate_share_text_badge:
    "This badge is certified by the Bitcoin blockchain!"
    "¡Esta insignia está certificada en la blockchain de Bitcoin!";
  public_certificate_share_text_default:
    "This document is certified by the Bitcoin blockchain!"
    "¡Este documento está certificado en la blockchain de Bitcoin!";
  abridged_title_diploma:
    "Summarized digital diploma"
    "Diploma digital abreviado";
  abridged_title_attendance:
    "Summarized digital certificate of attendance"
    "Certificaido de asistencia digital abreviado";
  abridged_title_badge:
    "Summarized digital badge"
    "Insignia digital abreviada";
  abridged_title_default:
    "Summarized certified digital document"
    "Documento digital certificado abreviado";
  abridged_lead_text:
    "This is a summarized presentation with a preview of some details."
    "Esta es una presentación abreviada que permite previsualizar algunos datos.";
  abridged_verify_diploma:
    "Verify them by visiting the full original diploma."
    "Verifíquelos visitando el diploma original completo.";
  abridged_verify_attendance:
    "Verify them by visiting the full original certificate of attendance."
    "Verifíquelos visitando el certificado de asistencia original completo.";
  abridged_verify_badge:
    "Verify them by visiting the full original badge."
    "Verifíquelos visitando la insignia original completa.";
  abridged_verify_default:
    "Verify them by visiting the full original certified document."
    "Verifíquelos visitando el documento certificado original completo.";
  abridged_signed_by:
    "Signed by"
    "Firmado por";
  abridged_stamped_on:
    "Date of certification by Constata"
    "Fecha de certificación por Constata";
  abridged_diploma_zip_name:
    "Summarized diploma in english and spanish"
    "Diploma abreviado en inglés y español";
  abridged_attendance_zip_name:
    "Summarized certificate of attendance in english and spanish"
    "Certificado de asistencia abreviado en inglés y español";
  abridged_badge_zip_name:
    "Summarized badge in english and spanish"
    "Insignia abreviada en inglés y español";
  abridged_document_zip_name:
    "Summarized document in english and spanish"
    "Documento certificado abreviado en inglés y español";
  diploma_schema_name:
    "Name"
    "Nombre y apellido";
  diploma_schema_email:
    "Email"
    "Email";
  diploma_schema_recipient_identification:
    "Student identification"
    "Identificación del estudiante";
  diploma_schema_custom_text:
    "Attaining the degree of"
    "Recibiendo el título de";
  diploma_schema_motive:
    "Completed the course of"
    "Completó la curricula de";
  diploma_schema_date:
    "Graduation date"
    "Fecha de graduación";
  diploma_schema_place:
    "Place"
    "Lugar";
  diploma_schema_shared_text:
    "Closing remarks"
    "Palabras de cierre";
  attendance_schema_name:
    "Name"
    "Nombre y apellido";
  attendance_schema_email:
    "Email"
    "Correo electrónico";
  attendance_schema_recipient_identification:
    "Attendee identification"
    "Identificación del asistente";
  attendance_schema_custom_text:
    "Title company or affiliation"
    "Título, empresa, afiliación";
  attendance_schema_motive:
    "Name of the event"
    "Nombre del evento";
  attendance_schema_date:
    "Event date"
    "Fecha";
  attendance_schema_place:
    "Place"
    "Lugar";
  attendance_schema_shared_text:
    "Additional remarks"
    "Información adicional";
  badge_schema_name:
    "Name"
    "Nombre y apellido";
  badge_schema_email:
    "Email"
    "Email";
  badge_schema_recipient_identification:
    "Recipient identification"
    "Identificación del receptor";
  badge_schema_custom_text:
    "Additional description"
    "Descirpción adicional";
  badge_schema_motive:
    "Achievement"
    "Logro o mérito";
  badge_schema_date:
    "Date"
    "Fecha";
  badge_schema_place:
    "Place"
    "Lugar";
  badge_schema_shared_text:
    "Final Remarks"
    "Palabras de cierre";
}
