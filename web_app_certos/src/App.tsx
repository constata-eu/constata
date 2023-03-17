import {useEffect, useState} from "react";
import { Admin, Resource, CustomRoutes } from 'react-admin';
import { Route } from "react-router-dom";
import { DriveFileMove, Image, BackupTable } from '@mui/icons-material';
import { ApolloClient, InMemoryCache, HttpLink, from } from '@apollo/client';
import buildGraphQLProvider, { buildQuery } from 'ra-data-graphql-simple';
import authProvider from './components/auth_provider';
import { ConstataLayout } from './views/layout';
import Signup from './views/signup';
import VerifyEmail from './views/verify_email';
import Dashboard from './views/dashboard';
import Wizard from './views/wizard';
import Login from './views/login';
import { IssuanceList, IssuanceShow } from "./views/issuance";
import { TemplateList, TemplateShow } from "./views/template";
import gql from 'graphql-tag';
import polyglotI18nProvider from 'ra-i18n-polyglot';
import spanishMessages from 'ra-language-spanish';
import englishMessages from 'ra-language-english';
import { i18n as i18nMessages } from './i18n';
import constataTheme from './theme';
import KycRequest from './components/kyc_request';
import Loading from "./views/loading";
import {InvoiceLink, InvoiceLinkSuccess, InvoiceLinkError} from "./views/invoice_link";
import Safe from "./views/safe";
import { DownloadProofLink, CertificateShow } from "./views/download_proof_link";


function App() {
  const api_url = `${process.env.REACT_APP_CERTOS_API_DOMAIN || ''}/graphql/`
  const [dataProvider, setDataProvider] = useState<any>(null);

  useEffect(() => {
    async function initApp() {
      const httpLink = new HttpLink({
        uri: api_url,
        fetch: async (url, req: any, ...more) => {
          return await navigator.locks.request("only_one_request_at_a_time", async () => {
            await authProvider.injectAuthorization(url, req);
            return (await fetch(url, req, ...more));
          })
        }
      });

      const client = new ApolloClient({
        link: from([ httpLink ]),
        cache: new InMemoryCache(),
      });

      const certosBuildQuery = introspection => (fetchType, resource, params) => {
        if (resource === 'SigningIterator') {

          const parser = function(data){
            if(data.data.data == null ){
              return { data: { done: true, id: true } };
            } else {
              return buildQuery(introspection)('GET_ONE', 'Entry', params).parseResponse(data);
            }
          }

          return {
            parseResponse: parser,
            variables: params.data,
            query: gql`mutation($input: SigningIteratorInput!){
              data: signingIterator(input: $input) {
                id,
                payload,
              }
            }`
          };
        } else if (resource === 'CreateIssuanceFromCsv') {
          const parser = function(data){
            return buildQuery(introspection)('GET_ONE', 'Issuance', params).parseResponse(data);
          }
          return {
            parseResponse: parser,
            variables: params.data,
            query: gql`mutation($input: CreateIssuanceFromCsvInput!){
              data: createIssuanceFromCsv(input: $input) {
                id
                templateId
                templateName
                templateKind
                state
                name
                createdAt
                errors
                tokensNeeded
                entriesCount
                adminVisitedCount
                publicVisitCount
              }
            }`
          };
        } else {
          return buildQuery(introspection)(fetchType, resource, params);
        }
      };

      const schema = (await (await fetch(`${api_url}introspect`)).json()).__schema;

      buildGraphQLProvider({ client, buildQuery: certosBuildQuery, introspection: {schema} }).then(provider => { setDataProvider(() => provider) });
    }

    initApp();
  }, [api_url]);

  if (!dataProvider) {
    return <Loading />
  }

  spanishMessages.ra.auth = {...spanishMessages.ra.auth, ...i18nMessages.es.patch.auth }
  spanishMessages.ra.action = {...spanishMessages.ra.action, ...i18nMessages.es.patch.action }
  spanishMessages.ra.navigation = {...spanishMessages.ra.navigation, ...i18nMessages.es.patch.navigation }

  const messages = {
    es: { ...spanishMessages, ...i18nMessages.es },
    en: { ...englishMessages, ...i18nMessages.en },
  };
  const i18nProvider = polyglotI18nProvider(
    locale => messages[locale],
    navigator.language.startsWith('es') ? 'es' : 'en'
  );

  return (
    <Admin
      disableTelemetry
      dashboard={Dashboard}
      dataProvider={dataProvider}
      authProvider={authProvider}
      i18nProvider={i18nProvider}
      loginPage={Login}
      theme={constataTheme}
      layout={ConstataLayout}
     >
        <CustomRoutes noLayout >
          <Route path="/signup" element={<Signup />} />
          <Route path="/verify_email/:access_token" element={<VerifyEmail />} />
          <Route path="/invoice/:access_token" element={<InvoiceLink />} />
          <Route path="/invoices/muchas-gracias" element={<InvoiceLinkSuccess />} />
          <Route path="/invoices/error-al-pagar" element={<InvoiceLinkError />} />
          <Route path="/safe" element={<Safe />} />
          <Route path="/safe/:access_token" element={<DownloadProofLink />} />
          <Route path="/safe/:access_token/show" element={<CertificateShow />} />
        </CustomRoutes>
        <CustomRoutes>
          <Route path="/wizard" element={<Wizard />} />
          <Route path="/wizard/:id" element={<Wizard />} />
          <Route path="/request_verification" element={<KycRequest />} />
        </CustomRoutes>
        <Resource
          name="Issuance"
          list={IssuanceList}
          show={IssuanceShow}
          icon={DriveFileMove}
          options={{ label: 'Issuances' }}
        />
        <Resource
          name="Template"
          list={TemplateList}
          show={TemplateShow}
          icon={Image}
          options={{ label: 'Templates' }}
        />
        <Resource
          name="Entry"
          icon={BackupTable}
          options={{ label: 'Entries' }}
        />
    </Admin>
  );
}

export default App;
