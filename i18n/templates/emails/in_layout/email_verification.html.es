{% extends "emails/in_layout/_layout.html" %}
{% import "emails/_macros.html.es" as macros %}

{% block main %}
  Le escribimos porque solicitó utilizar esta casilla de correo electrónico.
  <br/><br/>
  Para hacerlo, necesitamos que confirme haber recibido este correo
  <br/><br/>
  <a style="padding: 10px; margin: 15px 0; background: #1059CE; color: #fafafa; text-decoration: none; font-weight: bold; text-transform: uppercase; font-size: 12px;" target="_blank" href="{{url_to_verify_email | safe }}">Confirmar recepción</a>
  <br/><br/>
  {{ macros::accept_tyc(url_to_tyc = url_to_tyc) }}
{% endblock main %}

{% block footer %}
  {% include "emails/_footer_outgoing.html.es" %}
{% endblock footer %}
