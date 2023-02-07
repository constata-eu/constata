{% extends "public_api/for_login/_base.html.tera" %}

{% block title %}Constata.eu | Crear Credenciales{% endblock title %}

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

{% block __back %}
  Volver
{% endblock __back %}

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

{% block other_body %}
  <div class="d-flex py-5 py-md-5 my-2 align-content-center flex-column flex-wrap container container-constata" >
    <div class="column g-6 bg-white text-dark rounded p-3">
        <h2 class="mb-3 font-weight-bold">¡Ups!</h2>
        <p><b>Este link ya fue utilizado para crear credenciales y no puede ser utilizado nuevamente.</b></p>
        <p><b>Puede gestionar su KYC <a class="to_kyc" href="" target="_blank">aqui.</b></a></p>
      </div>
  </div>
{% endblock other_body %}

{% block script %}
  document.querySelector(".to_kyc").href = window.location.origin + "/kyc_request";
{% endblock script %}
