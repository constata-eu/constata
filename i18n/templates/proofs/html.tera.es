{% import "proofs/_macros.tera" as macros %}
{% import "proofs/_macros_es.tera" as macros_es %}

<!DOCTYPE html>
<html lang="en">

<!--

¡Hola! 

Soy un desarrollador de constata, y voy a ayudarte a entender como se valida este sello de tiempo.
Esto que estás leyendo es el "código de fuente" del certificado.
Es como levantar el capot del coche y ver que hay debajo.
No te asustes de los símbolos como "<" o ">", intenta seguir leyendo.
Si en alguna parte te pierdes, o necesitas ayuda, puedes pedir ayuda a 
algún amigo informático, o contactarte conmigo en https://constata.eu

Este certificado de sello de tiempo contiene uno o más "Documentos" codificados con
el standard "base64". Puedes encontrar todos esos "Documentos" empezando en alguna parte
entre las lineas 250 a 300.

Cada "Documento" tiene una "huella digital" única, que se calcula con la función sha256sum.
Si bien la huella digital (fingerprint in inglés) es única, no revela ningúna información
privada acerca del "Documento" en si.

Constata agrupa las huellas digitales de varios "Documentos" y confecciona un "Boletín"
con todas ellas, luego, calcula la huella digital del "Boletín" y la publica en una
base de datos inmutable que es el Blockchain de Bitcoin. Puedes encontrar ese "Boletín"
completo en este certificado, justo después de que se listan todos los "documentos".

Si la huella digital de un "Documento" está en un "Boletín",
y la huella del "Boletín" está publicada desde una fecha en el Blockchain de Bitcoin,
podemos decir que el "Documento" existía desde antes de esa fecha y el sello es válido.
-->
<head>
<!-- La siguiente línea es informacion cosmética, puedes ignorarla -->
<style>{% include "proofs/_style.scss" %}</style> 

<script>
  async function constataValidation(){
    /*
    En este certificado de sello de tiempo pueden haber varios documentos,
    compuestos de una o más partes cada uno.
    Cada documento puede haber recibido el sello de tiempo en un momento diferente
    por lo que el sello de tiempo de cada uno se consulta en un boletín diferente.

    El contenido de esos "Boletines" y "Documentos" están codificados en este mismo certificado.

    El primer paso es verificar que todos los boletines contenidos en este certificado
    están respaldados por el blockchain de Bitcoin.

    Para hacer eso buscamos la huella digital de cada boletín en alguna de las varias
    copias públicas del blockchain de Bitcoin disponibles en internet.

    Este proceso puede fallar si el usuario no tiene internet, o si la mayoría de las
    copias del blockchain no están disponibles para consultarse.

    Si podemos verificar la presencia del boletín en al menos 2 copias lo damos por válido,
    y nos guardamos la fecha cierta que haya quedado registrada en el Blockchain para
    mostrarla en este certificaod.
    */

    writeLoadingDetails("Verificando boletines")

    const bulletins = document.querySelectorAll('.bulletin');
    const public_blockchain_copies = {{ explorers | json_encode(pretty = true) }};
    let bulletin_dates = {};

    for( bulletin of bulletins ) {
      const bulletin_id = bulletin.dataset.bulletinId;

      writeLoadingDetails(`Verificando boletin ${bulletin_id}`);

      const bulletin_fingerprint = await sha256sum( (new TextEncoder()).encode(bulletin.innerHTML) );
      const blockchain_transaction_id = bulletin.dataset.transactionHash;

      let blockchain_responses = [];

      for (url of public_blockchain_copies) {
        writeLoadingDetails(`Verificando boletin ${bulletin_id} en ${url}`);

        try {
          const response = await fetch(url+blockchain_transaction_id);
          if(response.ok){
            let json = await response.json();
            blockchain_responses.push({
              fingerprint: json.output?.[0].script_pubkey || json.vout[0].scriptpubkey || json.vout[0].scriptPubKey.hex,
              time: json.blocktime || json.status.block_time 
            });
            if (blockchain_responses.length > 1) {
              break
            }
          }
        } catch { }
      }

      if (blockchain_responses.length < 2) {
        return showBlockchainTemporarilyUnavailableMessage();
      }

      let bulletin_found_count = 0;

      for (response of blockchain_responses) {
        if(response.fingerprint.includes(bulletin_fingerprint)){
          bulletin_found_count += 1;
        }
      }

      if(bulletin_found_count < 2) {
        return showCorruptCertificateMessage();
      }

      bulletin_dates[bulletin_id] = new Date(blockchain_responses[0].time * 1000)
        .toLocaleString(undefined, { dateStyle: "medium", timeStyle: "long" });

      document.querySelectorAll(`.timestamp-${bulletin_id}`)
        .forEach(el => el.innerHTML = bulletin_dates[bulletin_id]);
    }

    /*
    Si todos los boletines son válidos, solo nos falta verificar que las huellas digitales
    de cada documento contenido en el certificado aparecen en su boletín correspondiente.

    Este proceso solo puede fallar porque el certificado fue adulterado de alguna forma.
    */

    const documents = document.querySelectorAll('.document');
    
    for( doc of documents ) {
      const bulletin = document.getElementById(`bulletin_${doc.dataset.bulletinId}`);
      const document_parts = doc.querySelectorAll('.document-part');
      writeLoadingDetails(`Verificando la integridad del documento ${doc.dataset.documentId}`);

      for (part of document_parts) {
        /*
        El contenido de cada archivo que forma parte de un documento
        se encuentra embebido en este certificado codificado en BASE64
        */
        const payload = base64ToBytes(part.querySelector(".payload").innerHTML);

        /* Como primer paso nos aseguramos que la huella del documento está en el boletín correspondiente */
        const part_fingerprint = await sha256sum(payload);

        if( !bulletin.innerHTML.includes(part_fingerprint) ){
          return showCorruptCertificateMessage();
        }

        /* Y luego, si hay firmas digitales, también las validamos con sus respectivos sellos de tiempo */
        const signature_elements = part.querySelectorAll('.digital-signature');

        for (element of signature_elements) {
          const signature = base64ToBytes(element.dataset.signature);
          const signer = element.dataset.signer;

          if(!bitcoinMessage.verify(payload, signer, signature)) {
            return showCorruptCertificateMessage();
          }

          const signature_fingerprint = await sha256sum(signature);
          const signature_bulletin = document.getElementById(`bulletin_${element.dataset.bulletinId}`);

          if(!signature_bulletin.innerHTML.includes(signature_fingerprint) ) {
            return showCorruptCertificateMessage();
          }
        }
      }   
    }
    return true;
  }

  window.onload = async function(){
    if(await ensure_running_on_secure_environment()){
      if(await constataValidation()){
        generatePreviews();
        document.getElementById("loader_overlay").style.display = "none";
      }
    }
  }

  function writeLoadingDetails(text){
    document.getElementById("loader_detail").innerHTML = text
  }
</script>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Certificado de Sello de tiempo Constata.eu</title>
</head>

<body>
<div id="loader_overlay" >
  <div id="loader_spin" class="hidden"></div>
  <p id="loader_detail">
    Para validar y visualizar este certificado
    <a href="{{ secure_origin }}/safe">
      por favor dirígete al
      entorno web seguro de Constata
    </a>.
    <br/>
    <br/>
    <br/>
    También puedes visualizarlo desde otro dispositivo, como tu ordenador.
  </p>
  <div id="message" class="hidden"></div>
</div>

<div id="mobile_warning" class="hidden">
  Este certificado funcionará mejor si lo abres en un ordenador.
</div>

<div class="wrapper">
<div class="watermark">
  <span>Certificado por</span> {{ macros::constata_svg_logo(color_one="BBBBBB", color_two="BBBBBB") }}
</div>

<!--
En esta parte del certificado almacenamos todos los "Documentos" y los "Boletines".
Es posiblemente la parte más larga del certificado.
Estos datos se usan en el algoritmo que vimos más arriba.
-->
{%- set doc_count = documents | length -%}
{%- set bulletin_count = bulletins | length -%}
{%- set bulletin_ids = bulletins | map(attribute="object") | map(attribute="id") | sort -%}

{%- if doc_count > 1 -%}
<div class="many-docs-notice">
  Contiene <strong>{{ doc_count }} documentos</strong> registrados
  {% if bulletin_count > 1 %}
    entre <strong class="timestamp-{{ bulletin_ids | first }}">{cargando fecha}</strong>
    y <strong class="timestamp-{{ bulletin_ids | last }}">{cargando fecha}</strong>.
  {% else %}
    el <strong class="timestamp-{{ bulletin_ids | first }}">{cargando fecha}</strong>.
  {% endif %}
</div>
{%- endif -%}

{% for doc in documents %}
  {%- set doc_index = loop.index0 -%}

  <div class="document" id="document_{{ loop.index0 }}" data-document-id="{{doc.id}}" data-bulletin-id="{{ doc.bulletin_id }}">
    {%- if doc_count > 1 %}
      <h3 class="document-header">Documento {{ loop.index }} de {{ doc_count }}</h3>
    {%- endif -%}

    <div class="previews"></div>

    {%- set part_count = doc.parts | length -%}

    {%- set base_part = doc.parts.0.object -%}
    {%- set size_in_mb = (base_part.size_in_bytes / 1024 / 1024) | round(method="ceil", precision=2) -%}
    {% if part_count > 1 %}
      <div class="document-index meta-section">
        <p>
          {% if base_part.content_type == "application/zip" %}
            Este documento es el fichero del tipo <strong>zip</strong>
            llamado <strong>{{ base_part.friendly_name }}</strong>
            que ocupa <strong>{{ size_in_mb }} MB</strong> y cuyos contenidos puede guardar a continuación.
          {% elif base_part.content_type == "message/rfc822" %}
            Este documento es un correo electrónico con asunto <strong>{{ base_part.friendly_name }}</strong>,
            que en total ocupa <strong>{{ size_in_mb }} MB</strong>, y cuyos contenidos puede guardar a continuación.
          {% else %}
            Índice de partes, puede guardarlas a continuación.
          {% endif %}
        </p>
        {% for part in doc.parts %}
          <div class="field field-{{ loop.index0 }}" >
            <strong>
              {% if part.object.is_base %}
                Documento completo:
              {% else %}
                Parte {{ loop.index0 }}:
              {% endif %}
            </strong>
            <a href="#" class="link-save" onclick="extractDocumentPart({{ doc_index }}, {{ loop.index0 }})">
              {{ part.object.friendly_name }} 
            </a>
          </div>
        {% endfor %}
      </div>
    {% else %}
      <div class="document-index meta-section">
        <p>
          Este documento es el fichero del tipo <strong>{{ base_part.content_type }}</strong>
          llamado <strong>{{ base_part.friendly_name }}</strong>
          que ocupa <strong>{{ size_in_mb }} MB</strong>
          que puede
          <a href="#" class="link-save" onclick="extractDocumentPart({{ doc_index }}, 0)">
            guardar tocando aquí.
          </a>
        </p>
      </div>
    {% endif %}
    
    {% for part in doc.parts %}
      <div
        id="document_part_{{doc_index}}_{{ loop.index0 }}"
        class="document-part"
        data-content-type="{{ part.object.content_type }}"
        data-hash="{{ part.object.hash }}"
        data-friendly-name="{{ part.object.friendly_name }}"
      >
        <div class="payload hidden">{{ part.contents }}</div> 

        {% if part.object.is_base %}
          <div class="signature">
            <div class="field">
              <b>Fecha de registro en blockchain de Bitcoin:</b>
              <span class="timestamp-{{ doc.bulletin_id }}"><i>{cargando fecha}</i></span>.
            </div>
          </div>
        {% endif %}

        {% for signature in part.object.signatures %}
          {% if persons_missing_kyc is containing(signature.person_id) %}
            <div class="not-verified">
              <strong>Identidad no verificada:</strong> La identidad legal del firmante no ha sido verificada por CONSTATA.
            </div>
          {% endif %}
          <div class="signature digital-signature" data-signature="{{signature.signature}}" data-signer="{{signature.pubkey_id}}" data-bulletin-id="{{ signature.bulletin_id }}" >
            <div class="field">
              <b>Firmado digitalmente por: </b>
              {{ macros_es::person_endorsements(person_id=signature.person_id, endorsements=endorsements[signature.person_id]) }}
            </div>
            <div class="field">
              <b>Firmado el:</b>
              <span class="timestamp-{{ signature.bulletin_id }}">{ cargando fecha }</span>
            </div>
            <div class="field">
              <b>Firma:</b>
              {{ signature.signature }}
            </div>
            <div class="field">
              <b>Clave Pública:</b>
              {{ signature.pubkey_id }}
            </div>
          </div>
        {% endfor %}

        {% set signers = part.object.signatures | map(attribute="person_id") %}
        {% if part.object.is_base and signers is not containing(doc.author_id) %}
          <div class="signature">
            <div class="field">
              <b>Remitido a Constata para su certificación por:</b>
              {{ macros_es::person_endorsements(person_id=doc.author_id, endorsements = endorsements[doc.author_id]) }}
            </div>
          </div>
        {% endif %}
      </div>
    {% endfor %}
  </div>
{% endfor %}

{% for bulletin in bulletins %}
  <div id="bulletin_{{bulletin.object.id}}" class="bulletin hidden" data-bulletin-id="{{bulletin.object.id}}" data-bulletin-hash="{{bulletin.object.hash}}" data-transaction-hash="{{ bulletin.object.transaction_hash }}">{{ bulletin.contents }}</div>
{% endfor %}

<div class="footer">
  {{ macros::constata_svg_logo() }}
  <p>
    Este es un certificado de sello de tiempo con validez legal y tecnológica,
    avalado por la existencia de datos específicos que solo pueden haber sido generados
    a partir de los documentos contenidos, y que fueron escritos con fecha cierta en una base de datos pública,
    distribuida e inmutable, conocida como Blockchain de Bitcoin.
  </p>
  <p>
    El certificado es este mismo fichero HTML que usted descargó. Puede compartirlo con terceros
    por el medio que prefiera, y ellos podrán validarlo de forma independiente.
  </p>

  <h2>Validación rápida via web</h2>
  <ol>
    <li>Ingrese a <a target="_blank" href="{{ secure_origin }}/safe">{{ secure_origin }}/safe</a>.</li>
    <li>Seleccione este mismo fichero HTML.</li>
    <li>Nuestra web analizará el fichero localmente en este dispositivo y lo mostrará nuevamente sólo si es válido.</li>
  </ol>

  <h2 id="independent_validation">Validación independiente con Blockchain de Bitcoin</h2>
  <p>
    La validación rápida asegura que este fichero fue generado por Constata y que la empresa avala su contenido.
  </p>
  <p>
    En caso de que el servicio de Constata no se encuentre disponible, o el aval no fuera suficiente, se puede
    acceder a una auditoría detallada de este certificado <a id="expand_audit_log" onclick="expandAuditLog()" href="#independent_validation">tocando aquí</a>.
  </p>
</div>
</div><!-- /.wrapper -->

<div id="audit_log">
  <div class="wrapper">
    <h2>Auditoría detalla del certificado</h2>
    <p>
      A continuación se describen los procesos y tecnologías involucradas en la producción de este certificado.
      La terminología está disponible en el <a href="#glosario">Glosario</a> al pie.
    </p>
    <p>
      Para dejar evidencia de que los documentos contenidos en este certificado existieron en una fecha cierta
      <b>CONSTATA</b> confeccionó ficheros, denominados <b>BOLETINES</b>,
      que contienen los <b>HASHES</b> de cada <b>PARTE</b> de los <b>DOCUMENTOS</b> contenidos en este <b>CERTIFICADO</b>,
      y los <b>HASHES</b> de las <b>FIRMAS DIGITALES</b> aplicadas a algunas de esas <b>PARTES</b>.
    </p>
    <p>
      Luego, se dejó constancia del <b>HASH</b> de cada <b>BOLETÍN</b> en distintas <b>TRANSACCIONES BITCOIN</b>
      que fueron escritas en la base de datos distribuida <b>BLOCKCHAIN DE BITCOIN</b> en una fecha denominada <b>FECHA DEL BLOQUE</b>.
    </p>
    <p>
      Se establece entonces que los <b>DOCUMENTOS</b> y las <b>FIRMAS DIGITALES</b> aplicadas a los mismos existian al momento
      de la <b>FECHA DEL BLOQUE</b> del <b>BOLETÍN</b> que las contiene.
    </p>
    <p>
      Puede validarse que las <b>FIRMAS DIGITALES</b> aplicadas a los <b>DOCUMENTOS</b> son correctas de forma independiente,
      pero la asociación de esas firmas con las personas firmantes está avalada solamente por la firma de <b>CONSTATA</b>
      que fue aplicada a todo este fichero donde se recojen esa información.
    </p>
    <p>
      A diferencia de los sellos de tiempo en blockchain, para validar las firmas digitales es preciso obtener la clave
      pública de constata de una fuente segura, como nuestro sitio web.
      De otro modo, no podría estar seguro de que este certificado está firmado por Constata, y que la empresa avala
      los datos asociados a los firmantes.
      Una forma segura de obtener la clave pública de constata es en nuestro sitio web, aunque a lo largo de los años
      puede estar disponible via otros canales de distribución seguros.
    </p>

    <div class="section-1">
      <h3>Validación con Javascript</h3>
      <p>
        Este fichero contiene la rutina de validación de sellos de tiempo y firmas digitales implementada
        en el lenguaje Javascript, y puede verlo en el <a href="#" onclick="openSource()">código de fuente</a>.
        Cuando usted lo abrió en su navegador, se ejecutó dicha rutina, y como parte de la misma se consultaron
        distintas copias independientes de la blockchain de Bitcoin disponibles de forma pública en internet.
      </p>
      <p>
        El proceso fue exitoso, de otro modo, este certificado hubiera presentado un mensaje con el error correspondiente.
      </p>
    </div>

    <div class="section-1">
      <h3>Validación manual paso a paso</h3>
      <p>
        Si usted no está familiarizado con Javascrit, o prefiere realizar la validación de forma manual, estos son los pasos a seguir.
        Para reproducirlos se utilizará una terminal en un sistema operativo Linux, MacOS, o compatible.
      </p>
  
      <h4>Validar Boletines</h4>
      <p>
        Con este proceso, validamos que los <b>BOLETINES</b> fueron publicados en la <b>BLOCKCHAIN DE BITCOIN</b>
        en la fecha correspondiente.
        De esa forma, sabemos que todos los <b>DOCUMENTOS</b> referenciados existieron en esa fecha.
      </p>

      {% for bulletin in bulletins %}
        <div class="section-2" id="validate_bulletin_instructions_{{bulletin.object.id}}"> 
          <h4>Validar BOLETÍN #{{bulletin.object.id}}</h4>
          <ol>
            <li>
              <a href="#!" onclick="download_bulletin('boletin', `{{bulletin.object.id}}`)">Guarda el <b>BOLETIN</b> #{{bulletin.object.id}}</a> como un fichero local.
            </li>
            <li>
              <p>
                Calcula el <b>HASH</b> del mismo.
              </p>
              <pre class="simil-terminal">
                <code>$ shasum -a 256 /reemplazar/con/ruta/a/<span class="bulletin-filename">boletin_{{bulletin.object.id}}.txt</span></code>
                <code class="break-word bulletin-hash">{{bulletin.object.hash}}</code>
              </pre>
            </li>
            <li>
              <p>
                Consulta la <b>TRANSACCIÓN BITCOIN</b> para encontrar el <b>HASH</b> del <b>BOLETIN</b>.
                El identificador de la misma es <span class="break-word">{{ bulletin.object.transaction_hash }}</span>.
              </p>
              <div class="section-3">
                <h4>En Blockchain.com</h4>
                <ol>
                  <li>
                    <input type="checkbox"/>
                    Ingresa a ver
                    <a href="https://www.blockchain.com/es/btc/tx/{{ bulletin.object.transaction_hash }}" target="_blank">la transacción</a>.
                  </li> 
                  <li>
                    <input type="checkbox"/>
                    Observa la sección "Pkscript" para validar que contiene el <b>HASH</b>: <span class="bulletin-hash break-word">{{bulletin.object.hash}}</span>
                  </li>
                  <li>
                    <input type="checkbox"/> Observa que el campo <i>Sello de Tiempo</i> indica la fecha
                    <span>{{bulletin.object.block_time | date(format="%d-%B-%Y %H:%M")}}HS</span>.
                  </li>
                </ol>
              </div>
              <div class="section-3">
                <h4>En Mempool.space</h4>
                <ol>
                  <li>
                    <input type="checkbox"/>
                    Ingresa a ver
                    <a href="https://mempool.space/es/tx/{{ bulletin.object.transaction_hash }}" target="_blank">la transacción</a>.
                  </li>
                  <li><input type="checkbox"/> Despliega la sección "Detalles".</li>
                  <li><input type="checkbox"/> Valida que contiene el <b>HASH</b>: <span class="bulletin-hash break-word">{{bulletin.object.hash}}</span></li>
                  <li>
                    <input type="checkbox"/>
                    Observa que el campo <i>Sello de Tiempo</i> indica la fecha
                    <span>{{bulletin.object.block_time | date(format="%d-%B-%Y %H:%M")}}HS</span>.
                  </li>
                </ol>
              </div>
              <div class="section-3">
                <h4>En Blockstream.com</h4>
                <ol>
                  <li>
                    <input type="checkbox"/>
                    Ingresa a ver <a href="https://blockstream.info/tx/{{ bulletin.object.transaction_hash }}" target="_blank">la transacción</a>
                  </li>
                  <li><input type="checkbox"/> Despliega la sección "Detalles".</li>
                  <li><input type="checkbox"/> Valida que contiene el <b>HASH</b>: <span class="bulletin-hash break-word">{{bulletin.object.hash}}</span></li>
                  <li><input type="checkbox"/> Observa que el campo <i>Sello de Tiempo</i> indica la fecha <span>{{bulletin.object.block_time | date(format="%d-%B-%Y %H:%M")}}HS</span>.</li>
                </ol>
              </div>
            </li>
          </ol>
        </div>
      {% endfor %}

      <br/>
      <br/>
      <h3>Validación de los DOCUMENTOS</h3>
      <p>
        Con este proceso validamos que todos los <b>DOCUMENTOS</b> están siendo referenciados
        por alguno de los <b>BOLETINES</b> que validamos previamente, y por tanto, existieron
        en la fecha indicada por el <b>BOLETIN</b> correspondiente.
      </p>
      {% for doc in documents %}
        {% set part_count = doc.parts | length %}
        {% set doc_index = loop.index0 %}

        <div class="section-2" id="validate_document_{{doc.id}}">
          <h3>Validar DOCUMENTO #{{ doc_index + 1 }}</h3>
          <p>
            Este documento se compone de {{part_count}} {{ part_count | pluralize(singular="única parte", plural="partes") }}, 
            fue incluido en el BOLETÍN #{{ doc.bulletin_id }},
            con fecha <span class="timestamp-{{ doc.bulletin_id }}">{cargando fecha}</span>
          </p>

          {% for part in doc.parts %}
            <div class="section-3" id="validate_document_{{ doc_index }}_part_{{ loop.index0 }}">
              <ol>
                <li>
                  Guarda la parte <i>"{{ part.object.friendly_name }}"</i>
                  <a href="#!" onclick="extractDocumentPart({{ doc_index }}, {{ loop.index0 }})">
                    como un fichero local.
                  </a>
                </li>
                <li>
                  <p>
                    Calcula el <b>HASH</b> de la misma.
                  </p>
                  <pre class="simil-terminal">
                    <code>$ shasum -a 256 /reemplazar/con/ruta/a/<span>{{ doc_index + 1}}_{{ part.object.friendly_name }}</span></code>
                    <code class="part-hash">{{ part.object.hash }}</code>
                  </pre>
                </li>
                <li>
                  <p>
                    Busca el HASH de esta parte en el BOLETÍN #{{ doc.bulletin_id }} que guardaste previamente,
                    si el comando grep devuelve "1" es que el HASH está presente.
                  </p>

                  <pre class="simil-terminal">
                    <code>$ grep --count {{ part.object.hash }} /reemplazar/con/ruta/a/<span class="">boletin_{{ doc.bulletin_id }}.txt</span></code>
                    <code class="part-hash">1</code>
                  </pre>
                </li>
              </ol>
            </div>
          {% endfor %}
        </div>
      {% endfor %}
      <br/>
      <br/>
      {% include "proofs/_definitions.html.es" %}
    </div>
  </div><!-- /.wrapper -->
</div>

<script type="text/javascript">
  function isSecureEnvironment(){
    const l = window.location;
    if(l.protocol === "file:" || l.protocol == "content:" || l.origin === "{{ secure_origin }}" ){
      return true;
    }
  }

  function showCorruptCertificateMessage() {
    showErrorMessage(`
      <h1>⚠ CERTIFICADO INVÁLIDO</h1>
      <h2>El certificado de sello de tiempo está corrupto.</h2>
      <p>
        Eso puede significar que este certificado de sello de tiempo fue manipulado maliciosamente. 
        Para saber mas, contacte a <a href="https://constata.eu">Constata.eu</a> o consúltelo con un profesional de sistemas informáticos. El mecanismo de validación está descripto en este mismo certificado.
      </p>
    `);
  }

  function showBlockchainTemporarilyUnavailableMessage() {
    showErrorMessage(`
      <h1>⚠ NO SE PUDO VALIDAR EL CERTIFICADO</h1>
      <h2>Las consultas a la Blockchain de Bitcoin fallaron.</h2>
      <p>
        Esto puede ser un problema temporal con las copias públicas del blockchain, o con su conexión a internet. 
        Revise su conexión y espera unos minutos.
      </p>
      <p>
        Si continúa sin poder validar el sello y necesita ayuda, contacte a <a href="https://constata.eu">Constata.eu</a> o puede asesorarse con cualquier profesional de sistemas informáticos. El mecanismo de validación está descripto en este mismo certificado.
      </p>
    `);
  }

  async function ensure_running_on_secure_environment(){
    if(!isSecureEnvironment()) {
      showErrorMessage("Este certificado no puede verse en línea. Debes descargarlo a tu dispositivo para poder validarlo.<br/><br/>Iniciamos la descarga automáticamente.");
      const response = await fetch(document.location.href);
      const blob = await response.blob();
      save_locally(blob, 'certificado_constata.html');
      return false;
    }
    return true;
  }

  /*
  Estas son las funciones auxiliares invocadas por el algoritmo principal de validación.
  Se incluyen funciones desarrolladas por terceros con la licencia correspondiente.
  */
  {% include "proofs/_utils.js.tera" %}
</script>
</body>
</html>

