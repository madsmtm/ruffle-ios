// Helper to allow the storyboard to see the connections that we're making

#import <UIKit/UIKit.h>

@interface PlayerView: UIView
@end

@interface PlayerController: UIViewController
@end

@interface LibraryController : UITableViewController
@property IBOutlet PlayerView* logoView;
- (IBAction) toggleEditing: (UIBarButtonItem*) sender;
- (IBAction) editingClosed: (UIStoryboardSegue*) segue;
@end
