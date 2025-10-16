import { useState } from 'react';
import { Layout } from './components/layout/Layout';
import { TopBar } from './components/layout/TopBar';
import { Dashboard } from './components/views/Dashboard';
import { ModuleView } from './components/views/ModuleView';
import { CreateEntityModal } from './components/views/CreateEntityModal';
import { useUIStore } from './stores/useUIStore';

const moduleNames: Record<string, string> = {
  dashboard: 'Dashboard',
  project: 'Project Management',
  requirements: 'Requirements',
  risks: 'Risk Management',
  design: 'Design & BOM',
  verification: 'V&V',
  manufacturing: 'Manufacturing',
};

const moduleEntityTypes: Record<string, 'task' | 'requirement' | 'risk'> = {
  project: 'task',
  requirements: 'requirement',
  risks: 'risk',
};

function App() {
  const { activeModule } = useUIStore();
  const title = moduleNames[activeModule] || 'Tessera';
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);

  const handleNewEntity = () => {
    setIsCreateModalOpen(true);
  };

  const entityType = moduleEntityTypes[activeModule] || null;

  return (
    <Layout>
      <TopBar
        title={title}
        onNewEntity={activeModule !== 'dashboard' ? handleNewEntity : undefined}
      />

      <div className="flex-1 overflow-auto">
        {activeModule === 'dashboard' && <Dashboard />}
        {activeModule === 'project' && <ModuleView module="project" />}
        {activeModule === 'requirements' && <ModuleView module="requirements" />}
        {activeModule === 'risks' && <ModuleView module="risks" />}
        {activeModule === 'design' && <ModuleView module="design" />}
        {activeModule === 'verification' && <ModuleView module="verification" />}
        {activeModule === 'manufacturing' && <ModuleView module="manufacturing" />}
      </div>

      <CreateEntityModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        entityType={entityType}
      />
    </Layout>
  );
}

export default App;
