{% extends "public_api/terms_acceptance/_for_acceptance_base.html.tera" %}

{% block title %}Constata.eu | Términos y Condiciones{% endblock title %}

{% block __confirm_text %}
Confirmo que he leído y acepto los Términos y Condiciones de uso.
{% endblock __confirm_text %}

{% block __confirm %}
Confirmar
{% endblock __confirm %}

{% block __reject %}
Rechazar
{% endblock __reject %}

{% block __reject_footer %}
Rechazar
{% endblock __reject_footer %}

{% block __accept %}
Aceptar
{% endblock __accept %}

{% block __cannot_use %}
No podrá utilizar nuestros servicios hasta que acepte los Términos y Condiciones.
{% endblock __cannot_use %}

{% block __back %}
Volver
{% endblock __back %}

{% block __back_from_error %}
Volver
{% endblock __back_from_error %}

{% block __loading %}
Cargando...
{% endblock __loading %}

{% block __an_error_ocurred %}
Ha ocurrido un error, recargue la página y vuelva a intentarlo.
{% endblock __an_error_ocurred %}

{% block __include_terms %}
{% include "public_api/terms_acceptance/_index.html.tera.es" %}
{% endblock __include_terms %}

{% block __terms_were_accepted %}
Ha aceptado nuestros Términos y Condiciones,
puede cerrar la página.
{% endblock __terms_were_accepted %}
