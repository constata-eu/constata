{% extends "public_api/documents/_cannot_delete_base.html.tera" %}

{% block title %}Constata.eu | Descartar Documento{% endblock title %}

{%- block __cannot_delete_title -%}
Este documento no puede ser descartado.
{%- endblock __cannot_delete_title -%}

{%- block __cannot_delete_text -%}
El documento "{{ friendly_name }}" ya fue aceptado, y por este motivo no es posible desistir de certificarlo.
{%- endblock __cannot_delete_text -%}
