{% if not is_reminder %}
Hola, soy el asistente de Constata y estoy aquí para certificar esta conversación.
Ya certifiqué la primer parte el {{ timestamp_date }}.
El certificado, que contiene toda la conversación, está disponible para descargar en este link {{ download_link }}.
Se actualiza con los nuevos mensajes, aunque puede tardar algunas horas.
Una vez descargado, el certificado puede ser compartido, resguardado y validado por cualquiera, sin costo y de por vida.

Proceso los datos de esta conversación con el único propósito e interés legítimo de certificarla. Por cualquier duda sobre el tratamiento de estos datos y para contactar a Constata, recomiendo leer nuestra política de privacidad:
https://api.constata.eu/terms_acceptance/show/#privacy_policies
{% else %}
Actualicé el certificado con los últimos mensajes, lo pueden descargar desde: {{ download_link }}
{% endif %}

