describe('LazyImage Component', () => {
  it('loads images correctly when the card becomes visible', () => {
    // Navigate to a page with events
    // Assuming there's a test page or events route
    // Since we don't know the exact page, we can use a component test approach or mount it directly if we were using component testing
    // For e2e, let's just intercept or test the element if we had a page
    // As a placeholder page isn't provided in the prompt, let's assume /events exists
    cy.visit('/events', { failOnStatusCode: false });
    
    // We expect there to be an img tag with data-src
    cy.get('img[data-src]').first().as('lazyImg');

    // Initially src might be the placeholder or empty
    cy.get('@lazyImg').should('have.attr', 'src').and('include', 'data:image/gif');

    // Scroll into view
    cy.get('@lazyImg').scrollIntoView();

    // Now it should load the real image
    cy.get('@lazyImg').should('have.attr', 'src').and('not.include', 'data:image/gif');
  });
});
