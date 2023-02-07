{% extends "public_api/documents/_confirm_deletion_base.html.tera" %}

{% block title %}Constata.eu | Descartar Documento{% endblock title %}

{%- block __discard_parked_document -%}
Descartar Documento Detenido
{%- endblock __discard_parked_document -%}

{%- block __discard_parked_document_text -%}
Usted esta a punto de <strong>desistir en la certificación del documento "{{ friendly_name }}"</strong>. El documento no será certificado, lo borraremos de nuestros registros, y no se cargará a su cuenta. ¿Desea desistir de la certificación?
{%- endblock __discard_parked_document_text -%}

{%- block __discard_button_text -%}
Descartar
{%- endblock __discard_button_text -%}

{%- block __confirm_discard_title -%}
¿Confirma que desea eliminar este documento?
{%- endblock __confirm_discard_title -%}

{%- block __yes_desist -%}
Si, quiero desistir
{%- endblock __yes_desist -%}

{%- block __no_continue_certification -%}
No, continuar la certificación
{%- endblock __no_continue_certification -%}

{%- block __desisted_text -%}
Ha desistido de certificar este documento. Lo eliminamos de nuestros registros. Si cambia de opinión nuevamente y desea certificarlo, deberá remitirlo a Constata otra vez. Puede cerrar esta ventana.
{%- endblock __desisted_text -%}

{%- block __loading -%}
Cargando...
{%- endblock __loading -%}

{%- block __unexpected_error -%}
Ha ocurrido un error, recargue la página y vuelva a intentarlo. O contáctenos a <a href="mailto:hola@constata.eu">hola@constata.eu</a>.
{%- endblock __unexpected_error -%}

{%- block __back -%}
Volver
{%- endblock __back -%}
