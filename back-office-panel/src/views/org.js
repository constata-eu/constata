import React,{useState} from "react";
import { List, Datagrid, ShowButton, EditButton, TextField, TextInput, FunctionField,
         Show, NumberField, Edit, SimpleForm, TopToolbar, Tab, TabbedShowLayout,
         Button, ReferenceField, FilterForm, ReferenceManyField } from 'react-admin'
import { PostPagination, defaultSort, documentSort } from "../components/utils";
import {TopToolbarDefault} from "../components/top_toolbars";
import {OnlySaveToolbar} from "../components/bottom_toolbars";
import { useParams } from 'react-router-dom';
import { CardGiftcard, Link as LinkIcon, Delete, Image } from '@mui/icons-material';
import FilterTextInput from "../components/filter_textinput";
import MyUrlField from "../components/my_url_field";
import TokensAndCountField from "../components/tokens_and_count_field";
import TokensField from "../components/tokens_field";
import ParsedDateTextField from "../components/parsed_date_textfield";
import { personGrid } from "./person";
import { documentGrid, documentFilters } from "./document";
import { storyGrid } from "./story";
import { subscriptionGrid } from "./subscription";
import { paymentGrid } from "./payment";
import { invoiceGrid, invoiceFilters } from "./invoice";
import { invoiceLinkGrid, invoiceLinkFilters } from "./invoice_link";
import { giftGrid } from "./gift";
import { TemplateGrid } from "./template";



function OrgList() {

  const userFilters = [
    <FilterTextInput source="idEq" />,
    <FilterTextInput source="publicNameLike" />,
  ];

  return (
    <List
      empty={false}
      sort={defaultSort}
      filters={userFilters}
      perPage={20}
      pagination={<PostPagination />}
      actions={<TopToolbarDefault/>}
    >
      <Datagrid bulkActionButtons={false}>
        <TextField source='id' />
        <TextField source='publicName' />
        <TokensAndCountField source="fundedTokens" count="fundedCount" sortable={false} />
        <TokensAndCountField source="parkedTokens" count="parkedCount" sortable={false} />
        <TokensField source='tokenBalance' sortable={false} />
        <TokensField source='missingTokens' sortable={false} />
        <ShowButton />
        <EditButton />
      </Datagrid>
    </List>
  );
}

function OrgShow(){
  const { id } = useParams();
  const [documentFilter, setDocumentFilter] = useState({});
  const [invoiceFilter, setInvoiceFilter] = useState({});
  const [invoiceLinkFilter, setInvoiceLinkFilter] = useState({});

  const ListActionsRedirectAutocomplete = () => {
    return (
      <TopToolbar>
        {id && 
          <>
            <Button 
              href={`#/Gift/create?source={"orgId":"${id}"}`}
              label="resources.actions.createGift">
              <CardGiftcard/>
            </Button>
            <Button 
              href={`#/InvoiceLink/create?source={"orgId":"${id}"}`}
              label="resources.actions.createInvoiceLink">
              <LinkIcon/>
            </Button>
            <Button 
              href={`#/Template/create?source={"orgId":"${id}"}`}
              label="resources.actions.createTemplate">
              <Image/>
            </Button>
            <EditButton />
          </>
        }
      </TopToolbar>
    )
  };
  return (
    <Show actions={<ListActionsRedirectAutocomplete />} >
      <TabbedShowLayout syncWithLocation={false}>
        <Tab label="resources.actions.details" path="details">
          <NumberField source='id' />
          <TextField source='publicName' />
          <TokensField source='tokenBalance' />
          <TokensField source='missingTokens' />
          <TokensAndCountField source="totalDocumentTokens" count="totalDocumentCount" />
          <TokensAndCountField source="fundedTokens" count="fundedCount" />
          <TokensAndCountField source="parkedTokens" count="parkedCount" />
          <ParsedDateTextField source='firstParkedDate' />
          <ParsedDateTextField source='lastParkedDate' />
          <MyUrlField source='pendingInvoiceLinkUrl' />
          <ReferenceField source="subscriptionId" reference="Subscription" link="show">
            <TextField source="id" />
          </ReferenceField>
          <TextField source='stripeCustomerId' />
          <TextField source='logoUrl' />

          <FunctionField
            render={() => { 
              if (!id) return;
              return <Button 
                href={`#/OrgDeletion/create?source={"orgId":"${id}"}`}
                label="resources.actions.requestOrgDeletion">
                <Delete/>
              </Button>;
            }}
          />
        </Tab>
        <Tab label="resources.Person.many" path="persons">
          <div className="nested-resource" >
            <ReferenceManyField reference="Person" target="orgIdEq" label=""
              perPage={20}
              pagination={<PostPagination />}
            >
              {personGrid}
            </ReferenceManyField>
          </div>
        </Tab>
        <Tab label="resources.Document.many" path="documents">
          <div className="nested-resource" >
            <FilterForm
              filters={documentFilters}
              setFilters={setDocumentFilter}
              displayedFilters={ {"idLike": false} }
             />
            <ReferenceManyField reference="Document" target="orgIdEq" label=""
              sort={documentSort}
              filter={documentFilter}
              perPage={20}
              pagination={<PostPagination />}
            >
              {documentGrid}
            </ReferenceManyField>
          </div>
        </Tab>
        <Tab label="resources.Story.many" path="stories">
          <div className="nested-resource" >
            <ReferenceManyField reference="Story" target="orgIdEq" label=""
              sort={defaultSort}
              perPage={20}
              pagination={<PostPagination />}
            >
              {storyGrid}
            </ReferenceManyField>
          </div>
        </Tab>
        <Tab label="resources.Template.many" path="template">
          <div className="nested-resource" >
            <ReferenceManyField reference="Template" target="orgIdEq" label=""
              sort={defaultSort}
              perPage={20}
              pagination={<PostPagination />}
            >
              {TemplateGrid}
            </ReferenceManyField>
          </div>
        </Tab>
        <Tab label="resources.Subscription.one" path="subscription">
          <div className="nested-resource" >
            <ReferenceManyField reference="Subscription" target="orgIdEq" label=""
              sort={defaultSort}
              perPage={20}
              pagination={<PostPagination />}
            >
              {subscriptionGrid}
            </ReferenceManyField>
          </div>
        </Tab>
        <Tab label="resources.Payment.many" path="payments">
          <div className="nested-resource" >
            <ReferenceManyField reference="Payment" target="orgIdEq" label=""
              sort={defaultSort}
              perPage={20}
              pagination={<PostPagination />}
            >
              {paymentGrid}
            </ReferenceManyField>
          </div>
        </Tab>
        <Tab label="resources.Invoice.many" path="invoices">
          <div className="nested-resource" >
            <FilterForm
              filters={invoiceFilters}
              setFilters={setInvoiceFilter}
              displayedFilters={ {"idEq": false} }
             />
            <ReferenceManyField reference="Invoice" target="orgIdEq" label=""
              sort={defaultSort}
              filter={invoiceFilter}
              perPage={20}
              pagination={<PostPagination />}
            >
              {invoiceGrid}
            </ReferenceManyField>
          </div>
        </Tab>
        <Tab label="resources.InvoiceLink.many" path="invoiceLinks">
          <div className="nested-resource" >
            <FilterForm
              filters={invoiceLinkFilters}
              setFilters={setInvoiceLinkFilter}
              displayedFilters={ {"idEq": false} }
             />
            <ReferenceManyField reference="InvoiceLink" target="orgIdEq" label=""
              sort={defaultSort}
              filter={invoiceLinkFilter}
              perPage={20}
              pagination={<PostPagination />}
            >
              {invoiceLinkGrid}
            </ReferenceManyField>
          </div>
        </Tab>
        <Tab label="resources.Gift.many" path="gifts">
          <div className="nested-resource" >
            <ReferenceManyField reference="Gift" target="orgIdEq" label=""
              sort={defaultSort}
              perPage={20}
              pagination={<PostPagination />}
            >
              {giftGrid}
            </ReferenceManyField>
          </div>
        </Tab>
      </TabbedShowLayout>
    </Show>
  );
}

const OrgTitle = ({ record }) => {
  return <span>Org {record ? `"${record.publicName || record.id}"` : ''}</span>;
};

const OrgEdit = () => {
  return (
    <Edit title={<OrgTitle />} redirect="show">
      <SimpleForm toolbar={<OnlySaveToolbar />} warnWhenUnsavedChanges>
        <TextInput disabled source="id" />
        <TextInput source="logoUrl" autoComplete="off" />
        <TextInput source="publicName" autoComplete="off" />
      </SimpleForm>
    </Edit>
  )
};

export {OrgList, OrgShow, OrgEdit};
