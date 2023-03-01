import { useEffect, useState, useCallback } from 'react';
import { SelectInput, useDataProvider, useNotify, useTranslate } from 'react-admin';
import { handleErrors } from './utils';
import type { Validator } from 'react-admin';

interface SelectTemplatesInterface {
  source: string,
  validate?: Validator | Validator[],
  alwaysOn: boolean,
}

const SelectTemplates = ({source, validate, alwaysOn}: SelectTemplatesInterface) => {
  const dataProvider = useDataProvider();
  const notify = useNotify();
  const translate = useTranslate();
  const [choices, setChoices] = useState([{id: "", name: translate("certos.template.loading")}]);

  const reloadTemplateIdFilters = useCallback(async () => {
    try {
      let {data} = await dataProvider.getList('Template', {
        pagination: { page: 1, perPage: 200 },
        sort: {field: 'id', order: 'ASC'},
        filter: { archivedEq: false },
      });
      let list = data.map(template => {return { id: template.id, name: `${template.id} - ${template.name}` }});
      setChoices(list);
    } catch(e) {
      handleErrors(e, notify);
    }
  }, [dataProvider, notify]);

  useEffect(() => { reloadTemplateIdFilters() }, [reloadTemplateIdFilters]);

  return(
  <SelectInput source={source} validate={validate} alwaysOn={alwaysOn} choices={choices} />
)};

export default SelectTemplates;