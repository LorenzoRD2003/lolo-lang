// Representa el DAG de dependencias entre fases
// Nota: el diseño mas escalable posible seria
// in_degree: HashMap<&str, usize>
// adjacency: HashMap<&str, Vec<&str>>
// Pero es alto overkill para mis pocas fases de analisis semantico

use std::collections::HashSet;

use crate::semantic::phase::{
  CategoryCheckerPhase, CompileTimeConstantCheckerPhase, MutabilityCheckerPhase, NameResolverPhase,
  SemanticPhase, TypeCheckerPhase,
};

pub struct PhaseNode<'a> {
  pub phase: Box<dyn SemanticPhase<'a>>,
}

impl<'a> PhaseNode<'a> {
  pub fn new(phase: Box<dyn SemanticPhase<'a>>) -> Self {
    Self { phase }
  }
}

pub struct PhaseGraph<'a> {
  nodes: Vec<PhaseNode<'a>>,
  completed_phases: HashSet<&'static str>,
}

impl<'a> PhaseGraph<'a> {
  pub fn from(nodes: Vec<PhaseNode<'a>>) -> Self {
    Self {
      nodes,
      completed_phases: HashSet::new(),
    }
  }

  /// Indica cuales fases estan listas para ser ejecutadas, pero aun no fueron completadas.
  /// Por ejemplo, si `completed = { NameResolver }`, entonces devuelve `[TypeChecker, MutabilityChecker]`
  pub fn ready_phases(&self) -> Vec<&PhaseNode<'a>> {
    // es cuadratico pero no me importa. son pocos nodos. hacer un orden topologico aca es mucho laburo.
    self
      .nodes
      .iter()
      .filter(|node| {
        !self.completed_phases.contains(node.phase.name())
          && node
            .phase
            .dependencies()
            .iter()
            .all(|dep| self.completed_phases.contains(dep))
      })
      .collect()
  }

  /// Registra que una fase termino correctamente.
  /// Entonces, en la siguiente ejecucion del iterador, `ready_phases` puede detectar nuevas fases ejecutables.
  pub fn mark_phase_completed(&mut self, name: &'static str) {
    self.completed_phases.insert(name);
  }

  /// Indica si todas las fases terminaron correctamente.
  pub fn all_phases_completed(&self) -> bool {
    self.nodes.len() == self.completed_phases.len()
  }

  // Grafo de dependencias asociado al compilador
  // (fase1)
  // -> NameResolver
  // (fase2)
  // NameResolver -> {TypeChecker, MutabilityChecker, CompileTimeConstantChecker}
  // (fase3)
  // CompileTimeConstantChecker → CategoryChecker
  pub fn default_semantic_graph() -> Self {
    PhaseGraph::from(vec![
      PhaseNode::new(Box::new(NameResolverPhase)),
      PhaseNode::new(Box::new(TypeCheckerPhase)),
      PhaseNode::new(Box::new(MutabilityCheckerPhase)),
      PhaseNode::new(Box::new(CompileTimeConstantCheckerPhase)),
      PhaseNode::new(Box::new(CategoryCheckerPhase)),
    ])
  }
}
