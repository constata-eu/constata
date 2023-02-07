{% if give_welcome %}
Hola!
Soy el asistente virtual de Constata.eu. Yo certifico y resguardo tus conversaciones de telegram.
Si me escribes por privado certifico mensaje a mensaje.
Si creas un grupo y me invitas, certifico toda la conversación, quién dijo qué y cuándo, envíame algún mensaje y te lo sello.

No tengo ningún tipo de inteligencia, solo certifico mensajes y respondo a tus pedidos si dicen algo como "necesito ayuda" o "ayuda por favor".
Para cualquier consulta más compleja, puedes contactar a Constata en hola@constata.eu

{% if url_to_tyc %}
Proceso los datos que me indiques con el único propósito e interés legítimo de certificarlos,
es importante para mi que en algún momento leas y aceptes los términos y condiciones del servicio en este link:
{{ url_to_tyc }}
{% endif %}

{%- else -%}
Si necesitas hablar con algún humano de Constata, escribe por correo a hola@constata.eu, todavía no tienen telegram.
Aprovecho también para actualizarte sobre tu actividad:
Certificaste {{ funded_documents_count }} mensaje{{ funded_documents_count | pluralize }}.

{% if token_balance > 0 -%}
Tienes {{ token_balance }} Token{{ token_balance | pluralize }}.
{%- else -%}
No tienes Tokens a favor.
{%- endif -%}

{%- if subscription.max_monthly_gift -%}
Te puedo regalar hasta {{ subscription.max_monthly_gift }} Tokens por mes en la medida que los necesites.
{%- endif -%}

{%- if parked_count > 0 -%}
Tienes {{ parked_count }} mensaje{{ parked_count | pluralize }} detenido{{ parked_count | pluralize }}.

{%- if missing_tokens -%}
Necesitas comprar {{ missing_tokens }} token{{ missing_tokens | pluralize }} para certificarlos, te dejo el link aquí: {{ buy_tokens_link }}
{%- else -%}
Debes aceptar los Términos y Condiciones para que se procesen, aquí: {{ url_to_tyc }}
{%- endif -%}

{%- for parked_document in parked_documents_urls -%}
Para desistir de certificar "{{ parked_document[0] }}" visita {{ parked_document[0] }}.
{%- endfor -%}
{%- endif -%}

{%- for invoice in invoices -%}
Te recuerdo también que iniciaste una compra de Tokens via {{ invoice.payment_source }} y
puedes continuarla en {{ invoice.url }}
{%- endfor -%}
{%- endif -%}
