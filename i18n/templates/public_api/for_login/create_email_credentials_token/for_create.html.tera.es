{% extends "public_api/for_login/create_email_credentials_token/_for_create_base.html.tera" %}

{% block title %}Constata.eu | Crear Credenciales{% endblock title %}

{% block __welcome %}
<h2 class="mb-3 font-weight-bold">¡Te damos la bienvenida!</h2>
<p>Para autenticarse en nuestros servicios debe contar con su propia clave privada.</p>
<p>Esta clave nunca se envía a nuestros servidores y se almacena encriptada en su disco. Esto garantiza que usted y nadie más pueda firmar sus documentos.</p>
<p>¡Vamos a crear una ahora!</p>
{% endblock __welcome %}

{% block __create_credentials %}
Generar Credenciales
{% endblock __create_credentials %}

{% block __cancel %}
Cancelar
{% endblock __cancel %}

{% block __about_to_link_key %}
Está por vincular su email {{ email_address }} a su nueva clave privada.
{% endblock __about_to_link_key %}

{% block __enter_daily_password %}
Ingrese un password diario
{% endblock __enter_daily_password %}

{% block __daily_password %}
Contraseña diaria:
{% endblock __daily_password %}

{% block __confirm_daily_pass %}
Confirmar contraseña diaria:
{% endblock __confirm_daily_pass %}

{% block __passwords_must_match %}
Las contraseñas deben coincidir.
{% endblock __passwords_must_match %}

{% block __backup_passwords_must_match %}
Las contraseñas deben coincidir.
{% endblock __backup_passwords_must_match %}

{% block __continue %}
  Continuar
{% endblock __continue %}

{% block __continue_quit %}
  Continuar
{% endblock __continue_quit %}

{% block __continue_daily_pass %}
  Continuar
{% endblock __continue_daily_pass %}

{% block __enter_backup_password %}
  Ingrese una contraseña para la copia de respaldo
{% endblock __enter_backup_password %}

{% block __backup_password %}
Contraseña de la copia de respaldo:
{% endblock __backup_password %}

{% block __confirm_backup_pass %}
Confirmar contraseña de la copia de respaldo:
{% endblock __confirm_backup_pass %}

{% block __save %}
  Guardar
{% endblock __save %}

{% block __twelve_words %}
  12 Palabras
{% endblock __twelve_words %}

{% block __twelve_words_body %}
  <p>Escriba estas palabras en papel, en el orden en que se presentan.</p>
  <p>Son su semilla maestra. Las necesitará en caso de extraviar sus credenciales.</p>
  <p>NUNCA comparta estas palabras. Constata nunca te contactará para pedirlas.</p>
{% endblock __twelve_words_body %}

{% block __download_credentials %}
Descargar credenciales
{% endblock __download_credentials %}

{% block __sure_to_quit %}
<p>¿Está seguro que desea continuar?</p>
<p>Una vez salga de esta pestaña no podrá recuperar las 12 palabras.</p>
{% endblock __sure_to_quit %}

{% block __success %}
<p><b>¡Felicitaciones!</b></p>
<p>Ya ha descargado su archivo de credenciales.</p>
<p>Puede utilizarlo para autenticarse en nuestros servicios.</p>
<p>Ahora puede gestionar su KYC <a class="to_kyc" href="" target="_blank">aqui.</a></p>
{% endblock __success %}

{% block __you_have_logged_in %}
<h2 class="mb-3 font-weight-bold">¡Felicitaciones!</h2>
<p>Se ha logueado en nuestros servicios.</p>
<p>Ahora puede gestionar su KYC <a class="to_kyc" href="" target="_blank">aqui.</a></p>
{% endblock __you_have_logged_in %}

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
