{% macro hello_and_document(document_friendly_name) %}
  ¡Hola! Recibimos tu mensaje con el asunto <b><i>"{{document_friendly_name}}"</i></b>.
{% endmacro hello_and_document %}

{% macro accept_tyc(url_to_tyc) %}
  {% if url_to_tyc %}
    Además recuerde aceptar nuestros <b>Términos y condiciones</b>.
    <br/><br/>
    <a style="padding: 10px; margin: 15px 0; background: #1059CE; color: #fafafa; text-decoration: none; font-weight: bold; text-transform: uppercase; font-size: 12px;" target="_blank" href="{{url_to_tyc | safe }}">Revisar términos y condiciones</a>
    <br/>
  {% endif %}
{% endmacro accept_tyc %}

{% macro email_verification(url_to_verify_email, keep_private) %}
  {% if url_to_verify_email %}
    Necesitamos que confirme recepción de este correo.
    <br/><br/>
    <a style="padding: 10px; margin: 15px 0; background: #1059CE; color: #fafafa; text-decoration: none; font-weight: bold; text-transform: uppercase; font-size: 12px;" target="_blank" href="{{url_to_verify_email | safe }}">Confirmar recepción</a>
    <br/><br/>
  {% endif %}
{% endmacro hello_and_document %}

{% macro welcome_and_start(give_welcome, document_friendly_name, accepted, eta, url_to_tyc) %}
    {% if give_welcome %}
        Recuerda que cuando nos escribes o pones en copia certificamos ese correo.
        <br/><br/>
        También agregamos las respuestas al certificado, mientras se mantenga en copia de las mismas a <b>ace@constata.eu</b>.

    {% endif %}

    {% if accepted %}
        <br/><br/>
        El tiempo estimado de certificación es de {{ eta }} minutos. Lo recibirás en tu correo.
    {% elif not url_to_tyc %}
        <br/><br/>
        Antes de continuar debes obtener los tokens necesarios para la certificación.
    {% endif %}
{% endmacro welcome_and_start %}

{% macro terms_acceptance(has_enough_tokens, url_to_tyc, is_email_for_parked, parked_count) %}
    {% if url_to_tyc %}
        <br/>
        Para 
        {% if is_email_for_parked %}
            certificarlo{{ parked_count | pluralize }}
        {% else %}
            certificarlo
        {% endif %}
         debes aceptar nuestros <b>Términos y Condiciones</b>.
        <br/><br/>
        <a style="padding: 10px; margin: 15px 0; background: #1059CE; color: #fafafa; text-decoration: none; font-weight: bold; text-transform: uppercase; font-size: 12px;" target="_blank" href="{{url_to_tyc | safe }}">Aceptar Términos y Condiciones</a>
        <br/><br/>
        Una vez aceptados,
        {% if has_enough_tokens %}
            tu mensaje estará certificado en unos minutos y lo recibirás en tu correo.
        {% else %}
            para completar tu certificación deberás adquirir los tokens necesarios.
        {% endif %}
        {% if not is_email_for_parked %}
            <br/><br/>
        {% endif %}
    {% endif %}
{% endmacro terms_acceptance %}

{% macro cost_and_give_gift(gift, has_enough_tokens, cost, missing_tokens_for_other) %}
    <br/><br/>
    El costo de certificar este mensaje es de <b>{{ cost }} token{{ cost | pluralize }}</b>.
    {% if missing_tokens_for_other > 0  %}
        Además necesitas <b>{{ missing_tokens_for_other }} token{{missing_tokens_for_other | pluralize }}</b> para certificar todos los documentos que tienes pendientes.
    {% endif %}
    {% if gift %}
        <br/>
        <b>Te bonificamos {{ gift | round | int }} token{{ gift | pluralize }}</b>,
        {% if has_enough_tokens %}
            para que obtengas tu certificado sin cargo.
        {% else %}
            pero no fue suficiente.
        {% endif %}
        <br/><br/>
        <b>Costo del certificado = {{ cost | round | int }} token{{ cost | pluralize }}</b><br/>
        <b>Tokens bonificados = {{ gift | round | int }} token{{ gift | pluralize }}</b>
    {% endif %}
{% endmacro gift %}

{% macro enough_tokens(has_enough_tokens, missing_tokens, total_price, buy_tokens_link) %}
    {% if not has_enough_tokens %}
        <br/><br/>
        Puedes obtener los <b>{{ missing_tokens | round | int }} token{{ missing_tokens | pluralize }}</b> que faltan por <b>{{ total_price }} EUR</b>.
        <br/><br/>
        <a style="padding: 10px; margin: 15px 0; background: #1059CE; color: #fafafa; text-decoration: none; font-weight: bold; text-transform: uppercase; font-size: 12px;" target="_blank" href="{{buy_tokens_link | safe }}">Comprar Tokens</a>
        <br/><br/>
        Una vez que hayas comprado los tokens faltantes tu mensaje estará certificado en unos minutos y lo recibirás en tu correo.
    {% endif %}
{% endmacro enough_tokens %}

