{% extends "emails/_bare_layout.html" %}

{% block container  %}
  {% if custom_message %}
    <tr>
      <td style="padding:20px 30px 4px 30px;background-color:#fafafa;border-radius:11px 11px 11px 11px;border:1px solid #f0f0f5;border-color:rgba(201,201,207,.35);">
        <!--[if mso]>
        <table role="presentation" width="100%">
        <tr>
        <td style="width:100%;" align="left" valign="top">
        <![endif]-->
        <!--[if mso]>
        </td>
        <td style="width:395px;padding-bottom:20px;" valign="top">
        <![endif]-->
          {% if person_logo_url %}
            <img style="max-width: 200px; max-height: 300px;" src="{{ person_logo_url }}" />
          {% else %}
          <b>{{ on_behalf_of }}</b>
          {% endif %}
          <div style="margin: 20px 0;">
            {{ custom_message | escape | linebreaksbr | safe }}
          </div>
        <!--[if mso]>
        </td>
        </tr>
        </table>
        <![endif]-->
      </td>
    </tr>
    <tr>
      <td style="padding:10px;text-align:center;font-size:24px;font-weight:bold;"></td>
    </tr>
  {% endif %}
  <tr>
    <td style="padding:20px 30px 4px 30px;font-size:0;background-color:#fafafa;border-radius:11px 11px 0 0;border:1px solid #f0f0f5;border-color:rgba(201,201,207,.35);border-bottom:0;">
      <!--[if mso]>
      <table role="presentation" width="100%">
      <tr>
      <td style="width:100%;" align="left" valign="top">
      <![endif]-->
      <!--[if mso]>
      </td>
      <td style="width:395px;padding-bottom:20px;" valign="top">
      <![endif]-->
      {% if not custom_message %}
        <img src="https://constata.eu/assets/images/logo.png" style="max-width:70px;height:auto;border:none;text-decoration:none;color:#ffffff;">
      {% endif %}
      <div style="display:inline-block;width:100%;vertical-align:top;padding-bottom:20px;font-family:Inter, system-ui;font-size:15px;line-height:22px;color:#363636;">
        <p style="margin-top:0;margin-bottom:10px;">
          {% if custom_message %}
          {% else %}
            <br/>
            <br/>
          {% endif %}

          La empresa Constata.EU le transmite este mensaje
          de parte de <b>{{ on_behalf_of }}</b>, en referencia a
          un documento que certificamos.
          <br/>
          <br/>
          El documento está contenido en un <strong>Certificado</strong> con fecha
          <b>{{ timestamp_date }}</b> (horario UTC).
          <br/>
          <br/>
          <a style="padding: 10px; margin: 15px 0; background: #1059CE; color: #fafafa; text-decoration: none; font-weight: bold; text-transform: uppercase; font-size: 12px;" target="_blank" href="{{download_link}}">Descargar Certificado</a>
          <br/>
          <br/>
          El enlace de descarga es válido por 30 días. Una vez descargado, el <strong>certificado es válido de por vida</strong>.
          <br/><br/>
          Le recomendamos descargar y conservar el <b>Certificado</b>. Si fuera a presentarlo ante terceros, deberá remitirlo completo en formato digital para que puedan validar su autenticidad.

        </p>
      <!--[if mso]><i style="letter-spacing: 25px;mso-font-width:-100%">&nbsp;</i><![endif]-->
      </div>
      <!--[if mso]>
      </td>
      </tr>
      </table>
      <![endif]-->
    </td>
  </tr>
{% endblock container %}

{% block footer %}
  {% include "emails/_public_footer.html.es" %}
{% endblock footer %}
