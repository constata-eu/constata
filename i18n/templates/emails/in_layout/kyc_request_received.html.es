{% extends "emails/in_layout/_layout.html" %}
{% import "emails/_macros.html.es" as macros %}

{% block main %}
  Recibimos su pedido de verificación de identidad.
  <br/><br/>
  Lo revisaremos a al brevedad y le responderemos.
  <br/><br/>
  Si tiene preguntas acerca del proceso puede responder a este correo.
  <br/><br/>
  {{ macros::email_verification(url_to_verify_email = url_to_verify_email, keep_private = keep_private) }}
  {{ macros::accept_tyc(url_to_tyc = url_to_tyc) }}
{% endblock main %}

{% block footer %}
  {% include "emails/_footer_outgoing.html.es" %}
{% endblock footer %}
