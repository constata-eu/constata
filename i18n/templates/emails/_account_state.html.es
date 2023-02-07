    Estado de Cuenta<br/>
    Saldo: <strong>{{ token_balance }} Token{{ token_balance | pluralize }}.</strong><br/>

    {% if parked_count %}
      Mensajes pendientes: <strong>{{ parked_count }} detenido{{ parked_count | pluralize }}</strong>
      {% if missing_tokens %}
        (Necesita <strong>{{ missing_tokens + token_balance}} token{{ missing_tokens + token_balance | pluralize }}</strong> en total).
      {% else %}
         hasta aceptar los Términos y Condiciones.
      {% endif %}
      {% for parked_document in parked_documents_urls %}
      <br/>
        - <a target="_blank" href="{{ parked_document[1] }}">Descartar {{ parked_document[0] }}.</a>
      {% endfor %}
    {% else %}
      Tiene <strong>{{ funded_documents_count }} Documento{{ funded_documents_count | pluralize }} Sellado{{ funded_documents_count | pluralize }}.</strong>
    {% endif %}
    <br/>
    Mensajes certificados: <strong>{{ funded_documents_count }} Certificado{{ funded_documents_count | pluralize }}.</strong><br/>
    {% if parked_count %}
      Total de mensajes: <strong>{{ total_document_count }}.</strong><br/>
    {% endif %}

    {% if invoices %}
      <strong>Pagos pendientes</strong>:
      {% for invoice in invoices %}
          <br/>
           - <a target="_blank" href="{{ invoice.url }}">{{ invoice.id }} - {{ invoice.payment_source }}</a>
      {% endfor %}
      <br/>
    {% endif %}
