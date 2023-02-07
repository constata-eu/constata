{% extends "emails/in_layout/_layout.html" %}

{% block main %}
  ¡Hola! Este mensaje ha sido certificado el <b>{{ timestamp_date }}</b> (horario UTC).
  <br/><br/>
  <a style="padding: 10px; margin: 15px 0; background: #1059CE; color: #fafafa; text-decoration: none; font-weight: bold; text-transform: uppercase; font-size: 12px;" target="_blank" href="{{ download_link }}">Descargar Certificado</a>
  <br/><br/>
  Las respuestas al correo serán certificadas mientras se mantenga en copia de las mismas a ace@constata.eu.
  <br/><br/>
  El <b>certificado</b> se mantendrá actualizado con todas las respuestas, es posible descargar la última versión en el botón <b>Descargar Certificado</b>.
  <br/><br/>
  El <b>enlace de descarga caduca en 30 días</b>. Una vez descargado, el certificado es válido de por vida.
  <br/><br/>
  Recomendamos descargar y conservar el Certificado. Si fuera a presentarse ante terceros,
  debe remitirse completo en formato digital para una correcta validación de su autenticidad.
{% endblock main %}

{% block footer %}
  {% include "emails/_public_footer.html.es" %}
{% endblock footer %}
