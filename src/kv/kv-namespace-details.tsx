import { FunctionComponent } from 'react';
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb.tsx';
import { Location, useLocation, useNavigate } from 'react-router';
import { KvNamespace } from '@/kv/kv-models.ts';
import { KvTable } from '@/kv/table/kv-table.tsx';

const KvNamespaceDetails: FunctionComponent = () => {
  const location: Location<KvNamespace> = useLocation();
  const namespace = location.state;
  const navigate = useNavigate();

  if (!namespace) {
    navigate('/');
    return;
  }

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbPage>KV</BreadcrumbPage>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem>
              <BreadcrumbPage>{namespace.title}</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>

      <div className="flex flex-1 flex-col gap-4 p-4">
        <h2 className="pb-2 text-3xl font-semibold tracking-tight first:mt-0">
          {namespace.title}
          <br />
          <small className="text-sm text-gray-400">{namespace.id}</small>
        </h2>

        <KvTable namespace={namespace} />
      </div>
    </>
  );
};

export default KvNamespaceDetails;
