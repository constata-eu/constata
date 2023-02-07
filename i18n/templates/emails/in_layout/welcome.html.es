{% extends "emails/in_layout/_layout.html" %}
{% import "emails/_macros.html.es" as macros %}

{% block main %}
  Le damos la bienvenida a Constata.
  <br/><br/>
  Ya puede emitir Diplomas, Certificados de asistencia e Invitaciones.
  <br/><br/>
  {{ macros::email_verification(url_to_verify_email = url_to_verify_email, keep_private = keep_private) }}
  {{ macros::accept_tyc(url_to_tyc = url_to_tyc) }}
  {% if has_credentials %}
    Adjuntamos sus credenciales protegidas por contraseña, puede usarlas para acceder desde otros dispositivos.
    <br/><br/>
  {% endif %}
  Utilice sus credenciales sólo en nuestra web
  <a href="https://api.constata.eu">https://api.constata.eu</a>
  <br/><br/>
  Contáctenos de inmediato si cree que alguien más accedió a su cuenta.
  <br/>
{% endblock main %}

{% block footer %}
  {% include "emails/_footer_outgoing.html.es" %}
{% endblock footer %}
