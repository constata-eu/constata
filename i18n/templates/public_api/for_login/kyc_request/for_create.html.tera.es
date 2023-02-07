{% extends "public_api/for_login/kyc_request/_for_create_base.html.tera" %}

{% block title %}Constata.eu | Enviar Solicitud de Verificación de identidad{% endblock title %}

{%- block __main_title -%}
Solicitud de verificación
{%- endblock __main_title -%}

{%- block __fill_in_this_details -%}
Complete los siguientes datos:
{%- endblock __fill_in_this_details -%}

{%- block __name -%}
*Nombre:
{%- endblock __name -%}

{%- block __mandatory_field -%}
Este campo es obligatorio.
{%- endblock __mandatory_field -%}

{%- block __last_name -%}
*Apellido:
{%- endblock __last_name -%}

{%- block __mandatory_last_name -%}
Este campo es obligatorio.
{%- endblock __mandatory_last_name -%}

{%- block __document_type -%}
Tipo de Documento:
{%- endblock __document_type -%}

{%- block __document_number -%}
Número de Documento:
{%- endblock __document_number -%}

{%- block __birth_date -%}
Fecha de Nacimiento:
{%- endblock __birth_date -%}

{%- block __country -%}
País:
{%- endblock __country -%}

{%- block __nationality -%}
Nacionalidad:
{%- endblock __nationality -%}

{%- block __job -%}
Profesión:
{%- endblock __job -%}

{%- block __legal_entity_name -%}
Nombre de la Entidad Legal:
{%- endblock __legal_entity_name -%}

{%- block __legal_entity_country -%}
País de la Entidad Legal:
{%- endblock __legal_entity_country -%}

{%- block __legal_entity_registration -%}
Registro de la Entidad Legal:
{%- endblock __legal_entity_registration -%}

{%- block __legal_entity_tax_id -%}
Número de identificación tributario de la Entidad Legal:
{%- endblock __legal_entity_tax_id -%}

{%- block __documents -%}
*Documentación respaldatoria:
{%- endblock __documents -%}

{%- block __files_not_larger_than -%}
Debe seleccionar uno o varios archivos que no superen los 25MB.
{%- endblock __files_not_larger_than -%}

{%- block __info_for_files -%}
<p class="small">Debe subir al menos 3 archivos: frente y dorso del documento de identidad, y una selfie sosteniendo dicho documento.</p>
<p class="small">Si completa los campos sobre Entidad Legal deberá adjuntar además la documentación que lo acredita como representante.</p>
<p class="small">Los archivos deben estar en alguno de los siguientes formatos: .pdf, .jpg, .jpeg, .png</p>
{%- endblock __info_for_files -%}

{%- block __send -%}
Enviar
{%- endblock __send -%}

{%- block __info_about_mandatory_fields -%}
<p class="small">Los campos marcados con * son obligatorios.</p>
{%- endblock __info_about_mandatory_fields -%}

{%- block __confirm_body -%}
<p>¿Está seguro que desea continuar?</p>
<p>Una vez envíe su solicitud no podrá modificarla, y deberá esperar a que un administrador resuelva su pedido.</p>
{%- endblock __confirm_body -%}

{%- block __back -%}
Volver
{%- endblock __back -%}

{%- block __continue -%}
Continuar
{%- endblock __continue -%}

{% block __continue_quit %}
  Continuar
{% endblock __continue_quit %}

{% block __continue_daily_pass %}
  Continuar
{% endblock __continue_daily_pass %}

{%- block __finish_title -%}
Su solicitud ha sido enviada
{%- endblock __finish_title -%}

{%- block __finish_body -%}
  <p>Su pedido de KYC ha sido enviado correctamente.</p>
  <p>Un administrador verificará la información brindada y resolverá su pedido a la brevedad.</p>
  <p>Puede cerrar esta ventana.</p>
{%- endblock __finish_body -%}

{%- block __must_login_title -%}
Solicitud de verificación de identidad
{%- endblock __must_login_title -%}

{%- block __must_login_text -%}
Debe autenticarse con sus credenciales de Constata, o crear unas, para continuar.
{%- endblock __must_login_text -%}

{%- block __oops -%}
¡Ups!
{%- endblock __oops -%}

{%- block __oops_request_in_process -%}
Usted ya tiene una solicitud de KYC en proceso, espere a que sea aceptada o rechazada para enviar una nueva.
{%- endblock __oops_request_in_process -%}

{% block __enter_credentials %}
Ingrese sus credenciales
{% endblock __enter_credentials %}

{% block __select_credentials %}
Seleccione sus Credenciales
{% endblock __select_credentials %}

{% block __must_select_file %}
Debe seleccionar un archivo.
{% endblock __must_select_file %}

{% block __password %}
Contraseña:
{% endblock __password %}

{% block __daily_password %}
Contraseña:
{% endblock __daily_password %}

{% block __eight_chars_min %}
Mínimo 8 carácteres.
{% endblock __eight_chars_min %}

{% block __daily_eight_chars_min %}
Mínimo 8 carácteres.
{% endblock __daily_eight_chars_min %}

{% block __login %}
Ingresar
{% endblock __login %}

{% block __confirm_leave %}
¿Está seguro que desea cerrar sesión?
{% endblock __confirm_leave %}

{% block __logout %}
Cerrar sesión
{% endblock __logout %}

{% block __close %}
Cerrar
{% endblock __close %}

{% block __error_ocurred %}
Ha ocurrido un error.
{% endblock __error_ocurred %}

{% block __confirm %}
Confirmar
{% endblock __confirm %}

{% block __enter_password %}
Debe ingresar su password para continuar
{% endblock __enter_password %}

{% block __loading %}
Cargando...
{% endblock __loading %}

{%- block __link_already_used -%}
Este link ya fue utilizado para crear sus credenciales.
{%- endblock __link_already_used -%}

{%- block __logged_in_as -%}
Sesión iniciada como
{%- endblock __logged_in_as -%}
