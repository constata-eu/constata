{%- if group_chat_name -%}
  Voy a certificar los mensajes del chat grupal '{{ group_chat_name }}' a partir de ahora.
  (Si hubo mensajes antes, no puedo verlos).
{%- else -%}
  Voy a certificar tu mensaje.
{%- endif -%}

{% if url_to_tyc %}
Antes de seguir debes aceptar los términos y condiciones en este link: {{ url_to_tyc | safe }}.
{%- if not has_enough_tokens -%} También deberás pagar por la certificación. {%- endif -%}
{% elif not has_enough_tokens %}
Antes de continuar debes pagar por la certificación.
{% elif accepted %}
Lo tendré en unos {{ eta }} minutos.
{% endif %}
El costo de certificar este mensaje es de {{ cost | round | int }} token{{ cost | pluralize }}.
{% if missing_tokens_for_other > 0  -%}
Además necesitas {{ missing_tokens_for_other | round | int }} token{{missing_tokens_for_other | pluralize }} para certificar algunas conversaciones anteriores.
{% endif -%}
{% if gift -%}
Te regalé {{ gift | round | int }} token{{ gift | pluralize }}, {% if has_enough_tokens %}para que obtengas tu certificado sin cargo.{%- else -%}pero no fue suficiente.{%- endif -%}
{% endif -%}
{% if not has_enough_tokens %}
Puedes obtener los {{ missing_tokens | round | int }} token{{ missing_tokens | pluralize }} que faltan por {{ total_price }} EUR, visitando este link {{ buy_tokens_link | safe }}
{%- endif -%}

Escribe "ayuda" si necesitas algo más.
