import React, {useEffect, useState} from "react";
import { Admin, Resource, CustomRoutes } from 'react-admin';
import authProvider from './components/auth_provider';
import buildGraphQLProvider, { buildQuery } from 'ra-data-graphql-simple';
import {AdminUserList, AdminUserCreate} from "./views/admin_user";
import {BulletinList, BulletinShow} from './views/bulletin';
import {DocumentList, DocumentShow} from './views/document';
import {OrgList, OrgShow, OrgEdit} from './views/org';
import {EmailList, EmailShow} from './views/email_address';
import {GiftList, GiftShow, GiftCreate} from './views/gift';
import {InvoiceList, InvoiceShow} from "./views/invoice"
import {InvoiceLinkList, InvoiceLinkShow, InvoiceLinkCreate} from "./views/invoice_link";
import {KycRequestList, KycRequestShow, KycRequestEdit} from './views/kyc_request';
import {KycEndorsementList, KycEndorsementShow, KycEndorsementCreate,
        KycEndorsementEdit} from './views/kyc_endorsement';
import {PaymentList, PaymentShow} from './views/payment';
import {PersonList, PersonShow} from './views/person';
import {OrgDeletionList, OrgDeletionShow, OrgDeletionCreate}
        from './views/org_deletion';
import {PubkeyList, PubkeyShow} from './views/pubkey';
import {PubkeyDomainEndorsementList, PubkeyDomainEndorsementShow}
        from './views/pubkey_domain_endorsement';
import {StoryList, StoryShow} from './views/story';
import {SubscriptionShow, SubscriptionEdit}
        from "./views/subscription";
import {TermsAcceptanceList, TermsAcceptanceShow}
        from './views/terms_acceptance';
import {TemplateList, TemplateShow, TemplateCreate, TemplateEdit} from './views/template';
import MissingTokenList from './views/missing_token';
import TopTenList from './views/top_ten';
import {PermIdentity, PostAdd, Assignment, AttachMoney, VpnKey, Pause, Create, Fingerprint, Image,
        Link, Receipt, AccountCircle, DynamicFeed, DomainVerification, SupervisorAccount,
        CardGiftcard, AlternateEmail, EmojiEvents, Delete, Inbox, Business} from '@mui/icons-material';
import { Login } from 'ra-ui-materialui';
import { LoginForm } from './views/login.tsx';
import {
  ApolloClient,
  InMemoryCache,
  HttpLink,
  ApolloLink,
  from,
} from '@apollo/client';
import { onError } from '@apollo/client/link/error';
import MyLayout from './components/my_layout'
import polyglotI18nProvider from 'ra-i18n-polyglot';
import englishMessages from 'ra-language-english';
import * as adminMessages from './i18n';
import { AdminUserChangePassword } from "./views/change_password";
import { Route } from "react-router-dom";
import constataTheme from "./theme";
import gql from 'graphql-tag';


Login.defaultProps.children = <LoginForm />;

function App() {

  const api_url = `${process.env.REACT_APP_BACK_OFFICE_API || ''}/graphql/`
  const [dataProvider, setDataProvider] = useState(null);

  useEffect(() => {
    async function initApp() {
      const httpLink = new HttpLink({ uri: api_url });

      const authMiddleware = new ApolloLink((operation, forward) => {
        let authorization = authProvider.getToken() || null;
        operation.setContext(({ headers = {} }) => ({
          headers: {
          ...headers,
          "Authorization": authorization
          }
        }));
        
        return forward(operation);
      })

      const logoutLink = onError(({ networkError, graphQLErrors }) => {
        authProvider.checkError(networkError, graphQLErrors);
      })

      const client = new ApolloClient({
        link: from([
          authMiddleware,
          logoutLink,
          httpLink
        ]),
        cache: new InMemoryCache(),
      });
          
      const certosBuildQuery = introspection => (fetchType, resource, params) => {
        if (resource === 'PhysicalDeletion') {

          const parser = function(data){
            if(data.data.data == null ){
              return { data: { done: true, id: true } };
            } else {
              return buildQuery(introspection)('GET_ONE', 'OrgDeletion', params).parseResponse(data);
            }
          }

          return {
            parseResponse: parser,
            variables: params.data,
            query: gql`mutation($org_deletion_id:Int!){
              data: physicalDeletion(orgDeletionId: $org_deletion_id) {
                id,
                completed,
              }
            }`
          };

        } else {
          return buildQuery(introspection)(fetchType, resource, params);
        }
      };

      buildGraphQLProvider({ client, buildQuery: certosBuildQuery}).then(provider => { setDataProvider(() => provider) });
    }

    initApp();
  }, [api_url]);

  if (!dataProvider) {
    return <div> Loading </div>;
  }

  const messages = {
    en: { ...englishMessages, ...adminMessages.en },
  };
  const i18nProvider = polyglotI18nProvider(locale => messages[locale], "en");

  return (
    <Admin
      dataProvider={dataProvider}
      authProvider={authProvider}
      i18nProvider={i18nProvider}
      loginPage={Login}
      layout={MyLayout}
      theme={constataTheme}
      >
      {permissions => [
        <Resource
          name="Bulletin"
          list={BulletinList}
          show={BulletinShow}
          icon={Assignment}
          options={{ label: 'Bulletins' }}
        />,
        <Resource
          name="Document"
          list={DocumentList}
          show={DocumentShow}
          icon={PostAdd}
          options={{ label: 'Documents' }}
        />,
        <Resource
          name="Story"
          list={StoryList}
          show={StoryShow}
          icon={DynamicFeed}
          options={{ label: 'Stories' }}
        />,
        <Resource
          name="Org"
          list={OrgList}
          show={OrgShow}
          edit={OrgEdit}
          icon={Business}
          options={{ label: 'Organizations' }}
        />,
        <Resource
          name="Person"
          list={PersonList}
          show={PersonShow}
          icon={PermIdentity}
          options={{ label: 'Persons' }}
        />,
        <Resource
          name="TermsAcceptance"
          list={TermsAcceptanceList}
          show={TermsAcceptanceShow}
          icon={Create}
          options={{ label: 'Terms Acceptances' }}
        />,
        <Resource
          name="MissingToken"
          list={MissingTokenList}
          icon={Pause}
          options={{ label: 'Missing Tokens' }}
        />,
        <Resource
          name="EmailAddress"
          list={EmailList}
          show={EmailShow}
          icon={AlternateEmail}
          options={{ label: 'Emails' }}
        />,
        <Resource
          name="Pubkey"
          list={PubkeyList}
          show={PubkeyShow}
          icon={VpnKey}
          options={{ label: 'Public Keys' }}
        />,
        <Resource
          name="Template"
          list={TemplateList}
          show={TemplateShow}
          create={TemplateCreate}
          edit={TemplateEdit}
          icon={Image}
          options={{ label: 'Templates' }}
        />,
        <Resource
          name="Subscription"
          show={SubscriptionShow}
          edit={SubscriptionEdit}
          icon={AccountCircle}
          options={{ label: 'Subscriptions' }}
        />,
        <Resource
          name="Payment"
          list={PaymentList}
          show={PaymentShow}
          icon={AttachMoney}
          options={{ label: 'Payments' }}
        />,
        <Resource
          name="Invoice"
          list={InvoiceList}
          show={InvoiceShow}
          icon={Receipt}
          options={{ label: 'Invoices' }}
        />,
        <Resource
          name="InvoiceLink"
          list={InvoiceLinkList}
          show={InvoiceLinkShow}
          create={InvoiceLinkCreate}
          icon={Link}
          options={{ label: 'Invoice Links' }}
        />,
        <Resource
          name="Gift"
          list={GiftList}
          show={GiftShow}
          create={GiftCreate}
          icon={CardGiftcard}
          options={{ label: 'Gifts' }}
        />,
        <Resource
          name="TermsAcceptance"
          list={TermsAcceptanceList}
          show={TermsAcceptanceShow}
          icon={Create}
          options={{ label: 'Terms Acceptances' }}
        />,
        <Resource
          name="KycRequest"
          list={KycRequestList}
          show={KycRequestShow}
          edit={KycRequestEdit}
          icon={Inbox}
          options={{ label: 'Kyc Requests' }}
        />,
        <Resource
          name="KycEndorsement"
          list={KycEndorsementList}
          show={KycEndorsementShow}
          create={KycEndorsementCreate}
          edit={KycEndorsementEdit}
          icon={Fingerprint}
          options={{ label: 'Kyc Endorsement' }}
        />,
        <Resource
          name="PubkeyDomainEndorsement"
          list={PubkeyDomainEndorsementList}
          show={PubkeyDomainEndorsementShow}
          icon={DomainVerification}
          options={{ label: 'Pubkey Domain Endorsement' }}
        />,
        <Resource
          name="OrgDeletion"
          list={OrgDeletionList}
          show={OrgDeletionShow}
          create={OrgDeletionCreate}
          icon={Delete}
          options={{ label: 'Organization Deletions' }}
        />,

        <Resource
          name="TopTen"
          list={TopTenList}
          icon={EmojiEvents}
          options={{ label: 'Top 10' }}
        />,
        
        <CustomRoutes>
          <Route path="/ChangePassword" exact element={<AdminUserChangePassword />} />
        </CustomRoutes>,

        permissions === 'SuperAdmin'
          ? <Resource 
              name="AdminUser"
              list={AdminUserList}
              create={AdminUserCreate}
              icon={SupervisorAccount}
              options={{ label: 'Admin Users' }}
            />
          : null
      ]}
    </Admin>
  );
}

export default App;
