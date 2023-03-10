{% import "proofs/_macros.tera" as macros %}

<!DOCTYPE html>
<html lang="en">

<head>
<style>{% include "proofs/_style.scss" %}</style> 
<script type="text/javascript">
  {% include "proofs/_base_64_to_bytes.js.tera" %}
  {% include "proofs/_generate_previews.js.tera" %}
</script>
<script>
  window.onload = generatePreviews;
</script>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Certificado de Sello de tiempo Constata.eu</title>
</head>

<body>

<div class="wrapper">
<div class="watermark">
  <span> Ejemplo por </span>
  {{ macros::constata_svg_logo(color_one="BBBBBB", color_two="BBBBBB") }}
</div>

<div class="document">
  <div class="previews"></div>

  {%- set part_count = parts | length -%}

  {%- set base_part = parts.0 -%}
  {%- set size_in_mb = (base_part.size_in_bytes / 1024 / 1024) | round(method="ceil", precision=2) -%}
  {% if not has_kyc_endorsement %}
    <div class="not-verified">
      <strong>Identidad no verificada:</strong> La identidad legal del firmante no ha sido verificada por CONSTATA.
    </div>
  {% endif %}
  {% if part_count > 1 %}
    <div class="document-index meta-section">
      <p>
        {% if base_part.content_type == "application/zip" %}
          Este documento es el fichero del tipo
          <strong>zip</strong>
          llamado
          <strong>{{ base_part.friendly_name }}</strong>
          que ocupa
          <strong>{{ size_in_mb }} MB</strong>
          y cuyos contenidos puede guardar a continuación.
        {% else %}
          Índice de partes, puede guardarlas a continuación.
        {% endif %}
      </p>
      {% for part in parts %}
        <div class="field field-{{ loop.index0 }}" >
          <strong>
            {% if part.is_base %}
              Documento completo:
            {% else %}
              Parte
              {{ loop.index0 }}:
            {% endif %}
          </strong>
          <a href="#" class="link-save" onclick="alert('No se puede guardar en la vista previa')">
            {{ part.friendly_name }} 
          </a>
        </div>
      {% endfor %}
    </div>
  {% else %}
    <div class="document-index meta-section">
      <p>
        Este documento es el fichero del tipo
        <strong>{{ base_part.content_type }}</strong>
        llamado
        <strong>{{ base_part.friendly_name }}</strong>
        que ocupa
        <strong>{{ size_in_mb }} MB</strong>
        y puede
        <a href="#" class="link-save" onclick="alert('No se puede guardar en la vista previa')">
          guardarlo tocando aquí.
        </a>
      </p>
    </div>
  {% endif %}
  
  {% for part in parts %}
    <div
      id="document_part_0_{{ loop.index0 }}"
      class="document-part"
      data-content-type="{{ part.content_type }}"
      data-friendly-name="{{ part.friendly_name }}"
    >
      <div class="payload hidden">{{ part.payload }}</div> 

      {% if part.is_base %}
        <div class="signature">
          <div class="field">
            <b>
              Fecha de registro en blockchain de Bitcoin:
            </b>
            <span>
              Aquí pondremos la fecha y hora de registro de este documento.
            </span>.
          </div>
        </div>
        <div class="signature digital-signature">
          <div class="field">
            <b>
              Firmado digitalmente por:
            </b>
            En esta sección estarán los datos personales del firmante, como nombre, DNI, y empresa y cargo que ocupa.
          </div>
          <div class="field">
            <b>
              Firmado el:
            </b>
            Esta será la fecha y hora en la que se constató esta firma.
          </div>
          <div class="field">
            <b>
              Firma:
            </b>
            Aquí estará la firma digital única de este documento.
          </div>
          <div class="field">
            <b>
              Clave Pública:
            </b>
            Aquí estará la clave pública del firmante.
          </div>
        </div>
      {% endif %}
    </div>
  {% endfor %}
</div>

<button onclick="window.close()">&#x2715 Cerrar</button>

<div class="footer">
  {{ macros::constata_svg_logo() }}
  <p>
    Este es un ejemplo de certificado de sello de tiempo.
    En un certificado real, aquí estarán las instrucciones de validación.
  </p>
</div>
</div><!-- /.wrapper -->
</body>
</html>
